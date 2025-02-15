use core::fmt::Debug;

use bitfield::bitfield;
use embedded_can::{Id, StandardId};

#[cfg(not(feature = "async"))]
use embedded_hal::delay::DelayNs;
use embedded_hal::spi::Operation;
#[cfg(not(feature = "async"))]
use embedded_hal::spi::SpiDevice;

#[cfg(feature = "async")]
use embedded_hal_async::delay::DelayNs;
#[cfg(feature = "async")]
use embedded_hal_async::spi::SpiDevice;
#[cfg(feature = "async")]
use futures::future::OptionFuture;

use crate::memory::chip::{IoControlRegister, OscillatorControlRegister};
use crate::memory::controller::configuration::{
    CanControlRegister, DataBitTimeConfigurationRegister, NominalBitTimeConfigurationRegister,
    OperationMode, TimeStampControlRegister, TransmitterDelayCompensationMode,
    TransmitterDelayCompensationRegister,
};
use crate::memory::controller::fifo::{
    FifoControlRegister, FifoNumber, FifoStatusRegister, TxEventFifoControlRegister,
    TxEventFifoStatusRegister, TxQueueControlRegister, TxQueueStatusRegister, UserAddressKind,
    UserAddressRegister,
};
use crate::memory::controller::filter::{
    FilterControlRegister, FilterNumber, FilterObjectRegister, MaskRegister,
};
use crate::memory::controller::interrupt::{
    InterruptCodeRegister, InterruptRegister, RxInterruptStatusRegister,
    RxOverflowInterruptStatusRegister, TxAttemptInterruptStatusRegister, TxInterruptStatusRegister,
};
use crate::memory::{is_valid_ram_address, Register, RepeatedRegister, SFRAddress};
use crate::message::rx::{RxHeader, RxMessage};
use crate::message::tx::{TxEventObject, TxHeader, TxMessage};
use crate::message::{len_for_dlc, MAX_FD_BUFFER_SIZE};
use crate::settings::{
    self, BitTimeConfiguration, FilterConfiguration, FilterMatchMode, RxFifoConfiguration,
    TxFifoConfiguration,
};
use crate::settings::{
    FifoConfiguration, IoConfiguration, OscillatorConfiguration, Pll, SysClkDivider,
    TxEventFifoConfiguration, TxQueueConfiguration,
};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Failed to read from the chip over SPI
    SPIRead,
    /// Failed to write to the chip over SPI
    SPIWrite,
    /// Attempted to access an invalid RAM address
    InvalidRamAddress(u16),
    /// Tried to read data from ram that was not a multiple of 4 bytes
    InvalidReadLength(usize),
    /// Tried to write data to ram that was not a multiple of 4 bytes
    InvalidWriteLength(usize),
    /// Tried to transmit a message through the TXQ, but the TXQ is not enabled
    TxQueueDisabled,
    /// Tried to transmit a message with a FIFO not configured for transmission
    FifoNotTx,
    /// Tried to send a message that was too big for the FIFO
    FifoTooSmall,
    /// FIFO is already full and can not take any more messages
    FifoFull,
    /// Tried to read a message from a FIFO not configured for reception
    FifoNotRx,
    Other,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ConfigError {
    ChangeOpModeTimeout,
    ConfigurationModeTimeout,
    SPIFailedRAMEcho,
    PLLNotReady,
    Other(Error),
}

impl From<Error> for ConfigError {
    fn from(error: Error) -> Self {
        match error {
            Error::SPIRead | Error::SPIWrite => ConfigError::ConfigurationModeTimeout,
            _ => ConfigError::Other(error),
        }
    }
}

pub struct MCP2518FD<SPI> {
    spi: SPI,
}

#[cfg_attr(not(feature = "async"), maybe_async::maybe_async)]
impl<SPI, SPIE> MCP2518FD<SPI>
where
    SPI: SpiDevice<u8, Error = SPIE>,
    SPIE: Debug,
{
    /// Constructs a new MCP2518FD controller from an SPI bus and CS GPIO pin
    pub fn new(spi: SPI) -> MCP2518FD<SPI> {
        Self { spi }
    }

    /// Releases ownership of the SPI resources
    pub fn free(self) -> SPI {
        self.spi
    }

    /// Performs a software reset of the MCP2518FD chip over SPI (this puts it
    /// in configuration mode)
    pub async fn reset(&mut self) -> Result<(), Error> {
        let instruction = Instruction(OpCode::RESET);

        self.spi
            .write(&instruction.0.to_be_bytes())
            .await
            .map_err(|_| Error::SPIWrite)?;

        Ok(())
    }

    /// Does a full configuration sequence of the chip using the provided
    /// settings. This function puts the chip into configuration mode if it
    /// isn't already, verifies that SPI communication with the chip is
    /// working, and writes to all the necessary configuration registers.
    ///
    /// You may want to reset the chip before calling this method. See
    /// [`MCP2518FD::reset`] for more information.
    ///
    /// The data_bits_to_match field must be within 1..=18 if it is `Some`. A value of Some(0) will be interpreted the same as None, and
    pub async fn configure(
        &mut self,
        settings: settings::Settings,
        delay: &mut impl DelayNs,
    ) -> Result<(), ConfigError> {
        self.set_op_mode(OperationMode::Configuration, delay)
            .await
            .map_err(|_| ConfigError::ConfigurationModeTimeout)?;

        self.verify_spi_communications().await?;

        self.configure_osc(settings.oscillator, delay).await?;
        self.configure_io(settings.io_configuration).await?;
        self.configure_bit_timing(settings.bit_time_configuration)
            .await?;
        self.configure_tx_event_fifo(settings.tx_event_fifo).await?;
        self.configure_tx_queue(settings.tx_queue).await?;

        if settings.enable_time_based_counter {
            self.modify_register(|mut tscon: TimeStampControlRegister| {
                tscon.set_tbcen(true);
                tscon
            })
            .await?;
        }

        if let Some(dncnt) = settings.data_bits_to_match {
            self.modify_register(|mut cicon: CanControlRegister| {
                cicon.set_dncnt(dncnt);
                cicon
            })
            .await?;
        }

        self.modify_register(|mut cicon: CanControlRegister| {
            cicon.set_rtxat(true);
            cicon
        })
        .await?;

        self.modify_register(|mut ciint: InterruptRegister| {
            ciint.set_rxie(true);
            ciint.set_txie(true);

            if settings.enable_can_error_interrupts {
                ciint.set_ivmie(true);
                ciint.set_cerrie(true);
                ciint.set_serrie(true);
            }

            if settings.enable_spi_error_interrupt {
                ciint.set_spicrcie(true);
            }

            if settings.enable_ecc_error_interrupt {
                ciint.set_eccie(true);
            }

            // FIXME: expose a dedicated setting for this
            if settings.enable_time_based_counter {
                ciint.set_tbcie(true);
            }

            ciint
        })
        .await?;

        Ok(())
    }

    pub async fn get_op_mode(&mut self) -> Result<OperationMode, Error> {
        let c1con = self.read_register::<CanControlRegister>().await?;
        Ok(c1con.opmode())
    }

    /// Changes the operating mode of the chip. Will time out after 5 attempts.
    pub async fn set_op_mode(
        &mut self,
        op_mode: OperationMode,
        delay: &mut impl DelayNs,
    ) -> Result<(), ConfigError> {
        self.modify_register(|mut c1con: CanControlRegister| {
            c1con.set_opmode(op_mode);
            c1con
        })
        .await?;

        /* Delay 2ms checking every 500us for op mode change */

        const MAX_ATTEMPTS: usize = 5;

        for i in 0..MAX_ATTEMPTS {
            let c1con = self.read_register::<CanControlRegister>().await?;

            if c1con.opmode() == op_mode {
                break;
            } else if i == MAX_ATTEMPTS - 1 {
                return Err(ConfigError::ChangeOpModeTimeout);
            }

            delay.delay_us(500u32).await;
        }

        Ok(())
    }

    pub async fn configure_osc(
        &mut self,
        oscillator_settings: OscillatorConfiguration,
        delay: &mut impl DelayNs,
    ) -> Result<(), ConfigError> {
        self.modify_register(|mut osc: OscillatorControlRegister| {
            // If enabled, system clock comes from 10x PLL, otherwise it comes directly from the XTAL
            osc.set_pllen(match oscillator_settings.pll {
                Pll::On => true,
                Pll::Off => false,
            });

            // Whether or not to divide the system clock by 2
            osc.set_slckdiv(match oscillator_settings.divider {
                SysClkDivider::DivByOne => false,
                SysClkDivider::DivByTwo => true,
            });

            // Enable the clock
            osc.set_oscdis(false);

            osc
        })
        .await?;

        if let settings::Pll::On = oscillator_settings.pll {
            const MAX_ATTEMPTS: usize = 3;

            // Wait for PLL ready
            for i in 0..MAX_ATTEMPTS {
                let osc = self.read_register::<OscillatorControlRegister>().await?;

                if osc.pllrdy() {
                    break;
                } else if i == MAX_ATTEMPTS - 1 {
                    return Err(ConfigError::PLLNotReady);
                }

                delay.delay_us(500u32).await;
            }
        }

        Ok(())
    }

    pub async fn configure_io(&mut self, io_config: IoConfiguration) -> Result<(), ConfigError> {
        self.modify_register(|mut iocon: IoControlRegister| {
            iocon.set_xstbyen(io_config.enable_tx_standby_pin);
            iocon.set_txcanod(io_config.tx_can_open_drain);
            iocon.set_sof(io_config.start_of_frame_on_clko);
            iocon.set_intod(io_config.interrupt_pin_open_drain);
            iocon
        })
        .await?;

        Ok(())
    }

    pub async fn configure_bit_timing(
        &mut self,
        bit_time_config: BitTimeConfiguration,
    ) -> Result<(), ConfigError> {
        self.modify_register(|mut cinbtcfg: NominalBitTimeConfigurationRegister| {
            cinbtcfg.set_brp(bit_time_config.nominal.baud_rate_prescaler);
            cinbtcfg.set_tseg1(bit_time_config.nominal.time_segment_1);
            cinbtcfg.set_tseg2(bit_time_config.nominal.time_segment_2.value());
            cinbtcfg.set_sjw(bit_time_config.nominal.synchronization_jump_width.value());

            cinbtcfg
        })
        .await?;

        self.modify_register(|mut cidbtcfg: DataBitTimeConfigurationRegister| {
            cidbtcfg.set_brp(bit_time_config.data.baud_rate_prescaler);
            cidbtcfg.set_tseg1(bit_time_config.data.time_segment_1.value());
            cidbtcfg.set_tseg2(bit_time_config.data.time_segment_2.value());
            cidbtcfg.set_sjw(bit_time_config.data.synchronization_jump_width.value());

            cidbtcfg
        })
        .await?;

        self.modify_register(|mut citdc: TransmitterDelayCompensationRegister| {
            citdc.set_tdcmod(TransmitterDelayCompensationMode::Automatic);
            citdc.set_tdco(
                bit_time_config
                    .data
                    .transmitter_delay_compensation_offset
                    .value(),
            );
            citdc.set_tdcv(0);

            citdc
        })
        .await?;

        Ok(())
    }

    /// Enables/Disables the transmit event FIFO by setting C1CON.STEF and appropriate C1TEFCON bits.
    /// Be aware that fifo_size MUST be <= 32 and > 0, any other values will be clamped to 32.
    ///
    /// Also please keep in mind that the total RAM size is 2K and this code does absolutely
    /// zero validation that your configuration is under this limit. The documentation recommends
    /// configuring the TEF first, then TEQ, then FIFOs as necessary.
    pub async fn configure_tx_event_fifo(
        &mut self,
        tx_event_fifo_config: Option<TxEventFifoConfiguration>,
    ) -> Result<(), ConfigError> {
        self.modify_register(|mut c1con: CanControlRegister| {
            c1con.set_stef(tx_event_fifo_config.is_some());
            c1con
        })
        .await?;

        if let Some(config) = tx_event_fifo_config {
            self.modify_register(|mut tef_control: TxEventFifoControlRegister| {
                tef_control.set_fifo_size(config.fifo_size);

                tef_control.set_teftsen(config.enable_timestamps);

                tef_control.set_tefovie(config.enable_fifo_overflow_interrupt);
                tef_control.set_teffie(config.enable_fifo_full_interrupt);
                tef_control.set_tefhie(config.enable_fifo_half_full_interrupt);
                tef_control.set_tefneie(config.enable_fifo_not_empty_interrupt);

                tef_control
            })
            .await?;
        }

        Ok(())
    }

    /// Enables/Disables the transmit queue by setting C1CON.TXEN and appropriate C1TXQCON bits.
    /// Be aware that fifo_size MUST be <= 32 and > 0, any other values will be clamped to 32.
    ///
    /// Also please keep in mind that the total RAM size is 2K and this code does absolutely
    /// zero validation that your configuration is under this limit. The documentation recommends
    /// configuring the TEF first, then TEQ, then FIFOs as necessary.
    pub async fn configure_tx_queue(
        &mut self,
        tx_queue_config: Option<TxQueueConfiguration>,
    ) -> Result<(), ConfigError> {
        self.modify_register(|mut c1con: CanControlRegister| {
            c1con.set_txqen(tx_queue_config.is_some());
            c1con
        })
        .await?;

        if let Some(config) = tx_queue_config {
            self.modify_register(|mut tx_queue_control: TxQueueControlRegister| {
                tx_queue_control.set_retransmission_attempts(config.retransmission_attempts);
                tx_queue_control.set_txpri(config.message_priority);
                tx_queue_control.set_fifo_size(config.fifo_size);
                tx_queue_control.set_payload_size(config.payload_size);

                tx_queue_control.set_txatie(config.enable_transmit_attempts_exhausted_interrupt);
                tx_queue_control.set_txqeie(config.enable_queue_empty_interrupt);
                tx_queue_control.set_txqnie(config.enable_queue_not_full_interrupt);

                tx_queue_control
            })
            .await?;
        }

        Ok(())
    }

    /// Configures a FIFO based on the settings provided. As per documentation, a single FIFO must
    /// be dedicated to RX or TX and all objects in that queue must have the same payload size.
    pub async fn configure_fifo(
        &mut self,
        fifo_number: FifoNumber,
        fifo_config: FifoConfiguration,
    ) -> Result<(), Error> {
        self.modify_repeated_register(fifo_number, |mut fifo_control: FifoControlRegister| {
            fifo_control.set_fifo_size(fifo_config.fifo_size);
            fifo_control.set_payload_size(fifo_config.payload_size);

            match fifo_config.mode {
                settings::FifoMode::Transmit(TxFifoConfiguration {
                    priority,
                    retransmission_attempts,
                    enable_auto_rtr,
                    enable_transmit_attempts_exhausted_interrupt,
                    enable_fifo_empty_interrupt,
                    enable_fifo_half_empty_interrupt,
                    enable_fifo_not_full_interrupt,
                }) => {
                    fifo_control.set_txen(true);

                    fifo_control.set_txpri(priority);
                    fifo_control.set_retransmission_attempts(retransmission_attempts);
                    fifo_control.set_rtren(enable_auto_rtr);
                    fifo_control.set_txatie(enable_transmit_attempts_exhausted_interrupt);
                    fifo_control.set_tferffie(enable_fifo_empty_interrupt);
                    fifo_control.set_tfhrfhie(enable_fifo_half_empty_interrupt);
                    fifo_control.set_tfnrfnie(enable_fifo_not_full_interrupt);
                }
                settings::FifoMode::Receive(RxFifoConfiguration {
                    enable_message_timestamps,
                    enable_fifo_overflow_interrupt,
                    enable_fifo_full_interrupt,
                    enable_fifo_half_full_interrupt,
                    enable_fifo_not_empty_interrupt,
                }) => {
                    fifo_control.set_txen(false);

                    fifo_control.set_rxtsen(enable_message_timestamps);
                    fifo_control.set_rxovie(enable_fifo_overflow_interrupt);
                    fifo_control.set_tferffie(enable_fifo_full_interrupt);
                    fifo_control.set_tfhrfhie(enable_fifo_half_full_interrupt);
                    fifo_control.set_tfnrfnie(enable_fifo_not_empty_interrupt);
                }
            }

            fifo_control
        })
        .await?;

        Ok(())
    }

    /// Configures one of the 32 acceptance filters. If the filter_config is
    /// None, the filter will be disabled instead.
    ///
    /// Filters can be configured to accept only standard frames, only
    /// extended frames, or both standard and extended frames. If either the
    /// filter_bits or mask_bits fields are set as MessageId::Standard, the
    /// corresponding EID bits will be set to 0.
    ///
    /// When receiving standard frames, the EID component of the filter can be
    /// used to match against (up to) the first 18 bits of the message's data
    /// segment. The number of bits used is configured by `CiCON.DNCNT`. See
    /// the family reference manual for a more detailed description of this
    /// mechanism.
    pub async fn configure_filter(
        &mut self,
        filter_number: FilterNumber,
        filter_config: Option<FilterConfiguration>,
    ) -> Result<(), Error> {
        let (control_register_number, filter_index) = filter_number.get_control_register();

        // We need to disable the filter no matter what to configure it
        self.modify_repeated_register(
            control_register_number,
            |mut control: FilterControlRegister| {
                control.set_enabled(filter_index, false);
                control
            },
        )
        .await?;

        // If we are just disabling it, then we are done here
        let Some(filter_config) = filter_config else {
            return Ok(());
        };

        // Set filter object bits and filter mode
        self.modify_repeated_register(
            filter_number,
            |mut object_register: FilterObjectRegister| {
                match filter_config.filter_bits {
                    Id::Standard(id) => {
                        object_register.set_sid(id.as_raw());
                        object_register.set_eid(0);
                    }
                    Id::Extended(id) => {
                        object_register.set_sid(id.standard_id().as_raw());
                        object_register
                            .set_eid(id.as_raw() & ((!StandardId::MAX.as_raw() as u32) >> 11));
                    }
                }

                object_register.set_exide(match filter_config.mode {
                    FilterMatchMode::StandardOnly | FilterMatchMode::Both => false,
                    FilterMatchMode::ExtendedOnly => true,
                });

                object_register
            },
        )
        .await?;

        // Set the mask bits and exclusion mode
        self.modify_repeated_register(filter_number, |mut mask_register: MaskRegister| {
            match filter_config.mask_bits {
                Id::Standard(id) => {
                    mask_register.set_msid(id.as_raw());
                    mask_register.set_meid(0);
                }
                Id::Extended(id) => {
                    mask_register.set_msid(id.standard_id().as_raw());
                    mask_register
                        .set_meid(id.as_raw() & ((!StandardId::MAX.as_raw() as u32) >> 11));
                }
            }

            mask_register.set_mide(match filter_config.mode {
                FilterMatchMode::Both => false,
                FilterMatchMode::StandardOnly | FilterMatchMode::ExtendedOnly => true,
            });

            mask_register
        })
        .await?;

        // Set the BP and reenable the filter
        self.modify_repeated_register(
            control_register_number,
            |mut control: FilterControlRegister| {
                control.set_buffer_pointer(filter_index, filter_config.buffer_pointer);
                control.set_enabled(filter_index, true);
                control
            },
        )
        .await?;

        Ok(())
    }

    /* Transmit and Receive Functions */

    /// Pushes a new message into the TXQ without setting the TXREQ bit to
    /// request transmission.
    ///
    /// Use this function only if you need to queue multiple messages before
    /// transmitting all at once. To push a single message and immediately
    /// request transmission, use [`MCP2518FD::tx_queue_transmit_message`].
    pub async fn tx_queue_push_message(&mut self, message: &TxMessage) -> Result<(), Error> {
        /* Make sure TXQ is enabled */

        if !self.read_register::<CanControlRegister>().await?.txqen() {
            return Err(Error::TxQueueDisabled);
        }

        let mut control_register = self.read_register::<TxQueueControlRegister>().await?;

        /* Make sure FIFO is big enough */

        if control_register.payload_size().num_bytes() < message.data().len() {
            return Err(Error::FifoTooSmall);
        }

        /* Make sure FIFO is not full */

        let status_register = self.read_register::<TxQueueStatusRegister>().await?;

        if !status_register.txqnif() {
            return Err(Error::FifoFull);
        }

        /* Write message to RAM */

        let ram_address = self
            .read_repeated_register::<UserAddressRegister>(UserAddressKind::TxQueue)
            .await?
            .calculate_ram_address();

        let (length, bytes) = message.as_bytes();

        // We need to make sure that the data we are writing to ram has a length
        // which is a multiple of 4. By adding (4 - length % 4), we extend the
        // length to the next multiple 4 boundary. This isn't always a good
        // solution but in this specific case it works because we know that the
        // there is definitely at least that much ram allocated for the TX
        // message (we only do this when the DLC is < 8 and the minimum number
        // of bytes allocated for a TX message is 8)
        let data = &bytes[..length + (4 - length % 4)];

        self.write_ram(ram_address as u16, data).await?;

        /* Increment tail pointer but do NOT request transmission */

        control_register.set_uinc();

        self.write_register(control_register).await?;

        Ok(())
    }

    /// Requests transmission of all messages in the TXQ by setting the TXREQ
    /// bit.
    ///
    /// Use this function only if you already previously queued one or more
    /// messages with [`MCP2518FD::tx_queue_push_message`]. To push a single
    /// message and immediately request transmission, prefer
    /// [`MCP2518FD::tx_queue_transmit_message`].
    pub async fn tx_queue_request_transmission(&mut self) -> Result<(), Error> {
        self.modify_register(|mut txqcon: TxQueueControlRegister| {
            txqcon.set_txreq(true);
            txqcon
        })
        .await?;

        Ok(())
    }

    /// Pushes a message into the TXQ and immediately requests transmission by
    /// setting the TXREQ bit.
    ///
    /// To push multiple messages before requesting transmission, see
    /// [`MCP2518FD::tx_queue_push_message`] and
    /// [`MCP2518FD::tx_queue_request_transmission`].
    pub async fn tx_queue_transmit_message(&mut self, message: &TxMessage) -> Result<(), Error> {
        self.tx_queue_push_message(message).await?;
        self.tx_queue_request_transmission().await?;

        Ok(())
    }

    /// Pushes a new message into the given TX FIFO without setting the TXREQ
    /// bit to request transmission.
    ///
    /// Use this function only if you need to queue multiple messages before
    /// transmitting all at once. To push a single message and immediately
    /// request transmission, use [`MCP2518FD::tx_fifo_transmit_message`].
    pub async fn tx_fifo_push_message(
        &mut self,
        fifo_number: FifoNumber,
        message: &TxMessage,
    ) -> Result<(), Error> {
        let mut control_register = self
            .read_repeated_register::<FifoControlRegister>(fifo_number)
            .await?;

        /* Make sure it's a transmit FIFO */

        if !control_register.txen() {
            return Err(Error::FifoNotTx);
        }

        /* Make sure FIFO is big enough */

        if control_register.payload_size().num_bytes() < message.data().len() {
            return Err(Error::FifoTooSmall);
        }

        /* Make sure FIFO is not full */

        let status_register = self
            .read_repeated_register::<FifoStatusRegister>(fifo_number)
            .await?;

        if !status_register.tfnrfnif() {
            return Err(Error::FifoFull);
        }

        /* Write message to RAM */

        let ram_address = self
            .read_repeated_register::<UserAddressRegister>(UserAddressKind::Fifo(fifo_number))
            .await?
            .calculate_ram_address();

        let (length, bytes) = message.as_bytes();

        // We need to make sure that the data we are writing to ram has a length
        // which is a multiple of 4. By adding (4 - length % 4), we extend the
        // length to the next multiple 4 boundary. This isn't always a good
        // solution but in this specific case it works because we know that the
        // there is definitely at least that much ram allocated for the TX
        // message (we only do this when the DLC is < 8 and the minimum number
        // of bytes allocated for a TX message is 8)
        let data = &bytes[..length + (4 - length % 4)];

        self.write_ram(ram_address as u16, data).await?;

        /* Increment tail pointer but to NOT request transmission */

        control_register.set_uinc();

        self.write_repeated_register(fifo_number, control_register)
            .await?;

        Ok(())
    }

    /// Requests transmission of all messages in the given TX FIFO by setting
    /// the TXREQ bit.
    ///
    /// Use this function only if you already previously queued one or more
    /// messages with [`MCP2518FD::tx_fifo_push_message`]. To push a single
    /// message and immediately request transmission, prefer
    /// [`MCP2518FD::tx_fifo_transmit_message`].
    pub async fn tx_fifo_request_transmission(
        &mut self,
        fifo_number: FifoNumber,
    ) -> Result<(), Error> {
        self.modify_repeated_register(fifo_number, |mut fifocon: FifoControlRegister| {
            fifocon.set_txreq(true);
            fifocon
        })
        .await?;

        Ok(())
    }

    /// Pushes a message into the given TX FIFO and immediately requests
    /// transmission by setting the TXREQ bit.
    ///
    /// To push multiple messages before requesting transmission, see
    /// [`MCP2518FD::tx_fifo_push_message`] and
    /// [`MCP2518FD::tx_fifo_request_transmission`].
    pub async fn tx_fifo_transmit_message(
        &mut self,
        fifo_number: FifoNumber,
        message: &TxMessage,
    ) -> Result<(), Error> {
        self.tx_fifo_push_message(fifo_number, message).await?;
        self.tx_fifo_request_transmission(fifo_number).await?;

        Ok(())
    }

    /// Checks to see if there are any messages in the TEF
    pub async fn tx_event_fifo_has_next(&mut self) -> Result<bool, Error> {
        let status_register = self.read_register::<TxEventFifoStatusRegister>().await?;

        Ok(status_register.tefneif())
    }

    /// If there is a message available in the TEF it will be read but the FIFO
    /// tail pointer will **NOT** be incremented
    ///
    /// Unless you have a specific use case for this, you most likely want to
    /// use [`MCP2518FD::tx_event_fifo_get_next`]
    pub async fn tx_event_fifo_peek_next(&mut self) -> Result<Option<TxEventObject>, Error> {
        /* Make sure there is data to read */

        if !self.tx_event_fifo_has_next().await? {
            return Ok(None);
        }

        /* Get the address of the next object */

        let ram_address = self
            .read_repeated_register::<UserAddressRegister>(UserAddressKind::TxEventFifo)
            .await?
            .calculate_ram_address();

        /* Check if timestamps are enabled and read accordingly */

        let control_register = self.read_register::<TxEventFifoControlRegister>().await?;

        let obj = if control_register.teftsen() {
            let mut buf = [0u8; 12];

            self.read_ram(ram_address as u16, &mut buf).await?;

            TxEventObject {
                header: TxHeader([
                    u32::from_le_bytes(buf[0..4].try_into().unwrap()),
                    u32::from_le_bytes(buf[4..8].try_into().unwrap()),
                ]),
                timestamp: Some(u32::from_le_bytes(buf[8..12].try_into().unwrap())),
            }
        } else {
            let mut buf = [0u8; 8];

            self.read_ram(ram_address as u16, &mut buf).await?;

            TxEventObject {
                header: TxHeader([
                    u32::from_le_bytes(buf[0..4].try_into().unwrap()),
                    u32::from_le_bytes(buf[4..8].try_into().unwrap()),
                ]),
                timestamp: None,
            }
        };

        Ok(Some(obj))
    }

    /// If there is a message available in the TEF it will be read and the FIFO
    /// tail pointer will be incremented to allow for the next read operation
    ///
    /// To only check if a message is available without pulling it from the
    /// FIFO, see [`MCP2518FD::tx_event_fifo_has_next`] and
    /// [`MCP2518FD::tx_event_fifo_peek_next`]
    pub async fn tx_event_fifo_get_next(&mut self) -> Result<Option<TxEventObject>, Error> {
        let obj = self.tx_event_fifo_peek_next().await?;

        let Some(obj) = obj else {
            return Ok(None);
        };

        self.modify_register(|mut tefcon: TxEventFifoControlRegister| {
            tefcon.set_uinc();
            tefcon
        })
        .await?;

        Ok(Some(obj))
    }

    /// Checks to see if there are any messages in the given receive FIFO
    pub async fn rx_fifo_has_next(&mut self, fifo_number: FifoNumber) -> Result<bool, Error> {
        /* Make sure it's a receive FIFO */

        let control_register = self
            .read_repeated_register::<FifoControlRegister>(fifo_number)
            .await?;

        if control_register.txen() {
            return Err(Error::FifoNotRx);
        }

        /* Check is the FIFO has any messages in it */

        let status_register = self
            .read_repeated_register::<FifoStatusRegister>(fifo_number)
            .await?;

        Ok(status_register.tfnrfnif())
    }

    /// If there is a message available in the given RX FIFO it will be read,
    /// but the FIFO head pointer will **NOT** be incremented
    ///
    /// Unless you have a specific use case for this, you most likely want to
    /// use [`MCP2518FD::rx_fifo_get_next`]
    pub async fn rx_fifo_peek_next(
        &mut self,
        fifo_number: FifoNumber,
    ) -> Result<Option<RxMessage>, Error> {
        /* Make sure there is data to read */

        if !self.rx_fifo_has_next(fifo_number).await? {
            return Ok(None);
        }

        /* Get the address of the next object */

        let ram_address = self
            .read_repeated_register::<UserAddressRegister>(UserAddressKind::Fifo(fifo_number))
            .await?
            .calculate_ram_address();

        /* Read the message header to see how much data we need to read */

        let mut buf = [0u8; 8];

        self.read_ram(ram_address as u16, &mut buf).await?;

        let header = RxHeader([
            u32::from_le_bytes(buf[0..4].try_into().unwrap()),
            u32::from_le_bytes(buf[4..8].try_into().unwrap()),
        ]);

        /* Read timestamp (if applicable) */

        let control_register = self
            .read_repeated_register::<FifoControlRegister>(fifo_number)
            .await?;

        #[cfg(not(feature = "async"))]
        let timestamp = control_register
            .rxtsen()
            .then(|| {
                let mut ts = [0u8; 4];
                self.read_ram((ram_address + 4 * 2) as u16, &mut ts)?;

                Ok(u32::from_le_bytes(ts[..].try_into().unwrap()))
            })
            .transpose()?;

        #[cfg(feature = "async")]
        let timestamp = OptionFuture::from(control_register.rxtsen().then_some(async {
            let mut ts = [0u8; 4];
            self.read_ram((ram_address + 4 * 2) as u16, &mut ts).await?;

            Ok(u32::from_le_bytes(ts[..].try_into().unwrap()))
        }))
        .await
        .transpose()?;

        /* Read the content of the message */

        let mut data = [0u8; MAX_FD_BUFFER_SIZE];
        let data_len = len_for_dlc(header.dlc(), header.fdf()).unwrap();
        let data_offset = if timestamp.is_some() { 3 } else { 2 };

        // We need to make sure that the data we are reading from ram has a
        // length which is a multiple of 4. By adding (4 - length % 4), we
        // extend the length to the next multiple 4 boundary. This isn't always
        // a good solution but in this specific case it works because we know
        // that the there is definitely at least that much ram allocated for the
        // RX message (we only do this when the DLC is < 8 and the minimum
        // number of bytes allocated for a RX message is 8)
        let read_len = data_len + (4 - data_len % 4);

        self.read_ram(
            (ram_address + 4 * data_offset) as u16,
            &mut data[..read_len],
        )
        .await?;

        /* Assemble RxMessage */

        Ok(Some(
            RxMessage::new(header, timestamp, &data[..data_len]).unwrap(),
        ))
    }

    /// If there is a message available in the given RX FIFO it will be read,
    /// and the FIFO head pointer will be incremented to allow for the next
    /// read operation
    ///
    /// To only check if a message is available without pulling it from the
    /// FIFO, see [`MCP2518FD::rx_fifo_has_next`] and
    /// [`MCP2518FD::rx_fifo_peek_next`]
    pub async fn rx_fifo_get_next(
        &mut self,
        fifo_number: FifoNumber,
    ) -> Result<Option<RxMessage>, Error> {
        let msg = self.rx_fifo_peek_next(fifo_number).await?;

        let Some(msg) = msg else {
            return Ok(None);
        };

        self.modify_repeated_register(fifo_number, |mut tefcon: FifoControlRegister| {
            tefcon.set_uinc();
            tefcon
        })
        .await?;

        Ok(Some(msg))
    }

    /* Interrupt related operations */

    pub async fn get_highest_interrupt_codes(&mut self) -> Result<InterruptCodeRegister, Error> {
        self.read_register::<InterruptCodeRegister>().await
    }

    pub async fn get_top_level_interrupt_statuses(&mut self) -> Result<InterruptRegister, Error> {
        self.read_register::<InterruptRegister>().await
    }

    pub async fn get_rx_interrupt_statuses(&mut self) -> Result<RxInterruptStatusRegister, Error> {
        self.read_register::<RxInterruptStatusRegister>().await
    }

    pub async fn get_rx_overflow_interrupt_statuses(
        &mut self,
    ) -> Result<RxOverflowInterruptStatusRegister, Error> {
        self.read_register::<RxOverflowInterruptStatusRegister>()
            .await
    }

    pub async fn get_tx_interrupt_statuses(&mut self) -> Result<TxInterruptStatusRegister, Error> {
        self.read_register::<TxInterruptStatusRegister>().await
    }

    pub async fn get_tx_attempt_interrupt_statuses(
        &mut self,
    ) -> Result<TxAttemptInterruptStatusRegister, Error> {
        self.read_register::<TxAttemptInterruptStatusRegister>()
            .await
    }

    /* Generic register ops with mapping */

    pub async fn modify_repeated_register<R, F>(
        &mut self,
        index: R::Index,
        transform: F,
    ) -> Result<(), Error>
    where
        R: RepeatedRegister + From<u32> + Into<u32>,
        F: FnOnce(R) -> R,
    {
        let register = self.read_repeated_register::<R>(index).await?;

        self.write_repeated_register::<R>(index, transform(register))
            .await
    }

    pub async fn read_repeated_register<R>(&mut self, index: R::Index) -> Result<R, Error>
    where
        R: RepeatedRegister + From<u32>,
    {
        let address = R::get_address_for(index);

        self.read_sfr(&address).await.map(R::from)
    }

    pub async fn write_repeated_register<R>(
        &mut self,
        index: R::Index,
        value: R,
    ) -> Result<(), Error>
    where
        R: RepeatedRegister + Into<u32>,
    {
        let address = R::get_address_for(index);

        self.write_sfr(&address, value.into()).await
    }

    pub async fn modify_register<R, F>(&mut self, transform: F) -> Result<(), Error>
    where
        R: Register + From<u32> + Into<u32>,
        F: FnOnce(R) -> R,
    {
        let register = self.read_register::<R>().await?;

        self.write_register::<R>(transform(register)).await
    }

    pub async fn read_register<R>(&mut self) -> Result<R, Error>
    where
        R: Register + From<u32>,
    {
        let address = R::get_address();

        self.read_sfr(&address).await.map(R::from)
    }

    pub async fn write_register<R>(&mut self, value: R) -> Result<(), Error>
    where
        R: Register + Into<u32>,
    {
        let address = R::get_address();

        self.write_sfr(&address, value.into()).await
    }

    /* Raw SFR Ops (Minimal type checking) */

    async fn read_sfr(&mut self, address: &SFRAddress) -> Result<u32, Error> {
        let mut instruction = Instruction(OpCode::READ);
        instruction.set_address(*address as u16);

        let mut buf = [0u8; 4];

        self.spi
            .transaction(&mut [
                Operation::Write(&instruction.into_spi_data()),
                Operation::Read(&mut buf),
            ])
            .await
            .map_err(|_| Error::SPIRead)?;

        Ok(u32::from_le_bytes(buf))
    }

    async fn write_sfr(&mut self, address: &SFRAddress, value: u32) -> Result<(), Error> {
        let mut instruction = Instruction(OpCode::WRITE);
        instruction.set_address(*address as u16);

        self.spi
            .transaction(&mut [
                Operation::Write(&instruction.into_spi_data()),
                // The "instruction" needs to be converted to BE bytes but the actual SFR register
                // needs to be in LE format!!!
                Operation::Write(&value.to_le_bytes()),
            ])
            .await
            .map_err(|_| Error::SPIRead)?;

        Ok(())
    }

    /* RAM related functions */

    /// Verify SPI connection is working by writing to an available ram location.
    pub async fn verify_spi_communications(&mut self) -> Result<(), ConfigError> {
        let address = 0x400;
        for i in 0..32 {
            let data: u32 = 1 << i;
            self.write_ram(address, &data.to_le_bytes()).await?;

            let mut read_back_buf = [0u8; 4];
            self.read_ram(address, &mut read_back_buf).await?;
            let read_back_value = u32::from_le_bytes(read_back_buf);
            if read_back_value != data {
                return Err(ConfigError::SPIFailedRAMEcho);
            }
        }
        Ok(())
    }

    pub async fn verify_spi_communications_long(&mut self) -> Result<(), ConfigError> {
        let address = 0x400;

        let mut dword_data = [0u32; 32];
        for (i, d) in dword_data.iter_mut().enumerate() {
            *d = 1 << i;
        }

        let mut data = [0u8; 32 * 4];
        for i in 0..32 {
            let bytes = dword_data[i].to_le_bytes();

            data[i * 4] = bytes[0];
            data[i * 4 + 1] = bytes[1];
            data[i * 4 + 2] = bytes[2];
            data[i * 4 + 3] = bytes[3];
        }

        self.write_ram(address, &data).await?;

        let mut read_back_buf = [0u8; 32 * 4];
        self.read_ram(address, &mut read_back_buf).await?;

        if read_back_buf != data {
            return Err(ConfigError::SPIFailedRAMEcho);
        }

        Ok(())
    }

    /// Reads a contiguous range from RAM into the provided buffer
    pub async fn read_ram(&mut self, address: u16, data: &mut [u8]) -> Result<(), Error> {
        is_valid_ram_address(address as u32, data.len())
            .then_some(())
            .ok_or(Error::InvalidRamAddress(address))?;

        if data.len() % 4 != 0 {
            return Err(Error::InvalidReadLength(data.len()));
        }

        let mut instruction = Instruction(OpCode::READ);
        instruction.set_address(address);

        self.spi
            .transaction(&mut [
                Operation::Write(&instruction.into_spi_data()),
                Operation::Read(data),
            ])
            .await
            .map_err(|_| Error::SPIRead)?;

        Ok(())
    }

    /// Writes to a contiguous range in RAM from the provided buffer
    pub async fn write_ram(&mut self, address: u16, data: &[u8]) -> Result<(), Error> {
        is_valid_ram_address(address as u32, data.len())
            .then_some(())
            .ok_or(Error::InvalidRamAddress(address))?;

        if data.len() % 4 != 0 {
            return Err(Error::InvalidWriteLength(data.len()));
        }

        let mut instruction = Instruction(OpCode::WRITE);
        instruction.set_address(address);

        self.spi
            .transaction(&mut [
                Operation::Write(&instruction.into_spi_data()),
                Operation::Write(data),
            ])
            .await
            .map_err(|_| Error::SPIWrite)?;

        Ok(())
    }
}

/* Low level SPI instruction encoding */

bitfield! {
 struct Instruction(u16);
    impl Debug;
    u16;
    pub op_code, set_op_code: 15, 12;
    pub address, set_address: 11, 0;
}

impl Instruction {
    pub fn into_spi_data(self) -> [u8; 2] {
        self.0.to_be_bytes()
    }
}

struct OpCode;

impl OpCode {
    pub const RESET: u16 = 0b0000 << 12;
    pub const READ: u16 = 0b0011 << 12;
    pub const WRITE: u16 = 0b0010 << 12;
}
