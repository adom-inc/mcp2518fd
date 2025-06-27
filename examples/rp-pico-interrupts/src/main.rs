#![no_std]
#![no_main]

use core::cell::RefCell;

use cortex_m::interrupt::Mutex;
use defmt_rtt as _;
use embedded_can::{ExtendedId, Id, StandardId};
use embedded_hal_bus::spi::ExclusiveDevice;
use mcp2518fd::{
    memory::controller::{
        configuration::OperationMode,
        fifo::{FifoNumber, PayloadSize, HIGHEST_FIFO_PRIORITY},
        filter::FilterNumber,
        interrupt::RxInterruptFlagCode,
    },
    message::tx::TxMessage,
    settings::{
        BitTimeConfiguration, DataBitTimeConfiguration, FifoConfiguration, FifoMode,
        FilterConfiguration, FilterMatchMode, IoConfiguration, NominalBitTimeConfiguration,
        OscillatorConfiguration, RxFifoConfiguration, Settings, TxQueueConfiguration,
    },
    spi::MCP2518FD,
};
use panic_probe as _;
use rp_pico as bsp;

use defmt::info;
use fugit::RateExtU32;

use bsp::{
    entry,
    hal::{
        clocks::{init_clocks_and_plls, Clock},
        gpio::{
            bank0::{Gpio10, Gpio4, Gpio5, Gpio6, Gpio7},
            FunctionSioInput, FunctionSioOutput, FunctionSpi, Interrupt, Pin, PullDown, PullUp,
        },
        pac::{self, interrupt, SPI0},
        sio::Sio,
        spi::Enabled,
        watchdog::Watchdog,
        Spi, Timer,
    },
};

type CanIntPin = Pin<Gpio10, FunctionSioInput, PullUp>;
type CanSpiSclkPin = Pin<Gpio6, FunctionSpi, PullDown>;
type CanSpiMosiPin = Pin<Gpio7, FunctionSpi, PullDown>;
type CanSpiMisoPin = Pin<Gpio4, FunctionSpi, PullDown>;
type CanSpiCsPin = Pin<Gpio5, FunctionSioOutput, PullDown>;

type CanSpi = Spi<Enabled, SPI0, (CanSpiMosiPin, CanSpiMisoPin, CanSpiSclkPin)>;
type CanSpiDevice = ExclusiveDevice<CanSpi, CanSpiCsPin, Timer>;
type CanDriver = MCP2518FD<CanSpiDevice>;

static GLOBAL_CAN_INT_PIN: Mutex<RefCell<Option<CanIntPin>>> = Mutex::new(RefCell::new(None));
static GLOBAL_CAN_DRIVER: Mutex<RefCell<Option<CanDriver>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    info!("MCP2518FD Interrupts Example!");

    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let pins = rp_pico::hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let spi_sclk = pins.gpio6.into_function::<FunctionSpi>();
    let spi_mosi = pins.gpio7.into_function::<FunctionSpi>();
    let spi_miso = pins.gpio4.into_function::<FunctionSpi>();

    let spi_bus = Spi::<_, _, _, 8>::new(pac.SPI0, (spi_mosi, spi_miso, spi_sclk)).init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        200_000.Hz(),
        embedded_hal::spi::MODE_0,
    );
    let spi_cs = pins.gpio5.into_push_pull_output();

    let dev = ExclusiveDevice::new(spi_bus, spi_cs, timer).unwrap();

    let mut can = MCP2518FD::new(dev);

    /* Configure CAN  */

    {
        // Make sure the CAN controller gets reset (in case the Pico reboots
        // without the MCP2518FD losing power)
        can.reset().unwrap();

        // Configure the chip with some reasonable settings
        can.configure(
            Settings {
                // Standard for 40MHz XTAL
                oscillator: OscillatorConfiguration::default(),
                // Use default values for IOCON register
                io_configuration: IoConfiguration::new(),
                // Configure the bit timings (assumes a 40MHz input clock)
                bit_time_configuration: BitTimeConfiguration::new(
                    NominalBitTimeConfiguration::RATE_500_KBIT,
                    DataBitTimeConfiguration::RATE_2_MBIT,
                ),
                // Do not store any transmitted messages in the TEF
                tx_event_fifo: None,
                // Configure TXQ to have priority over all other FIFOs, and to
                // hold up to 8 messages with a max payload size of 32 bytes
                tx_queue: Some(TxQueueConfiguration::new(
                    HIGHEST_FIFO_PRIORITY,
                    8,
                    PayloadSize::Bytes32,
                )),
                // Enable the Time Based Counter (required for timestamps to be
                // recorded as non-zero)
                enable_time_based_counter: true,
                // Do not filter by any data bits
                data_bits_to_match: None,
                // Do not interrupt on CAN bus errors
                enable_can_error_interrupts: false,
                // Do not interrupt on SPI comms errors
                enable_spi_error_interrupt: false,
                // Do not interrupt on RAM ECC errors
                enable_ecc_error_interrupt: false,
            },
            &mut timer,
        )
        .expect("Failed to configure MCP2518FD");

        // Configure FIFO 1 as an RX FIFO to hold up to 16 messages with a max
        // payload size of 64 bytes and the "not empty" interrupt enabled
        can.configure_fifo(
            FifoNumber::Fifo1,
            FifoConfiguration {
                fifo_size: 16,
                payload_size: PayloadSize::Bytes64,
                mode: FifoMode::Receive(
                    RxFifoConfiguration::new()
                        .with_message_timestamps(true)
                        .with_fifo_not_empty_interrupt(true),
                ),
            },
        )
        .expect("Failed to configure FIFO 1 as RX");

        // Configure Filter 0 to accept all frame types (Standard or Extended),
        // with any message ID (mask is all 0s)
        can.configure_filter(
            FilterNumber::Filter0,
            Some(FilterConfiguration {
                buffer_pointer: FifoNumber::Fifo1,
                mode: FilterMatchMode::Both,
                filter_bits: Id::Extended(ExtendedId::ZERO),
                mask_bits: Id::Extended(ExtendedId::ZERO),
            }),
        )
        .expect("Failed to configure Filter 0 for FIFO 1");

        // Set controller to internal loopback mode (all transmitted messages
        // will be immediately received)
        can.set_op_mode(OperationMode::InternalLoopback, &mut timer)
            .expect("Failed to change chip operating mode");

        // Allow our CAN driver to be accessible from out interrupts
        cortex_m::interrupt::free(|cs| {
            GLOBAL_CAN_DRIVER.borrow(cs).replace(Some(can));
        });
    }

    /* Configure Interrupt Pin */

    {
        let can_int = pins.gpio10.reconfigure();

        can_int.set_interrupt_enabled(Interrupt::EdgeLow, true);

        cortex_m::interrupt::free(|cs| {
            GLOBAL_CAN_INT_PIN.borrow(cs).replace(Some(can_int));
        });

        unsafe {
            pac::NVIC::unmask(pac::Interrupt::IO_IRQ_BANK0);
        }
    }

    /* Send messages forever */

    let message = TxMessage::new_fd(
        Id::Standard(StandardId::MAX),
        &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
    )
    .expect("Message data is too long for frame kind (FD)")
    .with_bit_rate_switched(true);

    loop {
        // Send a message with the TXQ
        cortex_m::interrupt::free(|cs| {
            let mut can = GLOBAL_CAN_DRIVER.borrow(cs).borrow_mut();
            let can = can.as_mut().unwrap();

            can.tx_queue_transmit_message(&message)
                .expect("Failed to TX frame");
        });

        delay.delay_ms(500);
    }
}

#[interrupt]
fn IO_IRQ_BANK0() {
    static mut CAN_INT_PIN: Option<CanIntPin> = None;

    // Move our interrupt pin permanently into the interrupt handler
    if CAN_INT_PIN.is_none() {
        cortex_m::interrupt::free(|cs| {
            *CAN_INT_PIN = GLOBAL_CAN_INT_PIN.borrow(cs).take();
        });
    }

    let Some(int) = CAN_INT_PIN else {
        return;
    };

    // Don't respond to any spurious interrupts
    if !int.interrupt_status(Interrupt::EdgeLow) {
        return;
    }

    // Handle the interrupt and then clear it
    cortex_m::interrupt::free(|cs| {
        let mut can = GLOBAL_CAN_DRIVER.borrow(cs).borrow_mut();

        handle_can_driver_interrupt(can.as_mut().unwrap());

        int.clear_interrupt(Interrupt::EdgeLow);
    });
}

fn handle_can_driver_interrupt(can: &mut CanDriver) {
    // Check the interrupt status
    let codes = can
        .get_highest_interrupt_codes()
        .expect("Filed to get CAN interrupt codes");

    // Make sure we are responding to an RX interrupt. In this example we only
    // have 1 FIFO configured to receive messages, but in a real application we
    // should check which RX FIFO has the interrupt pending, what kind of
    // interrupt it is (by checking the FIFO control register), and if there
    // are any other interrupts also pending
    if codes.rx_code() != RxInterruptFlagCode::FifoInterrupt(FifoNumber::Fifo1) {
        return;
    }

    // Read the message back (we are in loopback mode)
    match can.rx_fifo_get_next(FifoNumber::Fifo1) {
        Ok(Some(frame)) => info!("Received frame {:?}", frame),
        Ok(None) => info!("No message to read!"),
        Err(e) => panic!("Error reading from FIFO: {:?}", e),
    }
}
