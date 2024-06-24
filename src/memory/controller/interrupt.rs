use bitfield::{bitfield, Bit};

use crate::{impl_register, impl_to_from_u32, software_clearable};

use super::{fifo::FifoNumber, filter::FilterNumber};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RxInterruptFlagCode {
    NoInterrupt,
    FifoInterrupt(FifoNumber),
    Reserved,
}

impl From<u8> for RxInterruptFlagCode {
    fn from(value: u8) -> Self {
        match value {
            0b0000_0001..=0b0001_1111 => Self::FifoInterrupt(value.try_into().unwrap()),
            0b0100_0000 => Self::NoInterrupt,
            0b0100_0001..=0b0111_1111 | 0b0010_0001..=0b0011_1111 | 0b0000_0000 => Self::Reserved,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TxInterruptFlagCode {
    NoInterrupt,
    TxqInterrupt,
    FifoInterrupt(FifoNumber),
    Reserved,
}

impl From<u8> for TxInterruptFlagCode {
    fn from(value: u8) -> Self {
        match value {
            0b0100_0000 => Self::NoInterrupt,
            0b0000_0000 => Self::TxqInterrupt,
            0b0000_0001..=0b0001_1111 => Self::FifoInterrupt(value.try_into().unwrap()),
            0b0100_0001..=0b0111_1111 | 0b0010_0001..=0b0011_1111 => Self::Reserved,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InterruptFlagCode {
    NoInterrupt,
    TransmitAttemptInterrupt,
    TransmitEventFifoInterrupt,
    InvalidMessageOccurred,
    OperationModeChangeOccurred,
    TbcOverflow,
    RxTxMabOverOrUnderflow,
    AddressErrorInterrupt,
    ReceiveFifoOverflowInterrupt,
    WakeUpInterrupt,
    ErrorInterrupt,
    FifoInterrupt(FifoNumber),
    TxqInterrupt,
    Reserved,
}

impl From<u8> for InterruptFlagCode {
    fn from(value: u8) -> Self {
        match value {
            0b0100_0000 => Self::NoInterrupt,
            0b0000_0000 => Self::TxqInterrupt,
            0b0000_0001..=0b0001_1111 => Self::FifoInterrupt(value.try_into().unwrap()),
            0b0010_0001..=0b0011_1111 | 0b0100_1011..=0b0111_1111 => Self::Reserved,
            0b0100_0001 => Self::ErrorInterrupt,
            0b0100_0010 => Self::WakeUpInterrupt,
            0b0100_0011 => Self::ReceiveFifoOverflowInterrupt,
            0b0100_0100 => Self::AddressErrorInterrupt,
            0b0100_0101 => Self::RxTxMabOverOrUnderflow,
            0b0100_0110 => Self::TbcOverflow,
            0b0100_0111 => Self::OperationModeChangeOccurred,
            0b0100_1000 => Self::InvalidMessageOccurred,
            0b0100_1001 => Self::TransmitEventFifoInterrupt,
            0b0100_1010 => Self::TransmitAttemptInterrupt,
            _ => unreachable!(),
        }
    }
}

bitfield! {
    pub struct InterruptCodeRegister(u32);
    impl Debug;
    u8;
    _icode, _: 6, 0;
    _filhit, _: 12, 8;
    _txcode, _: 22, 16;
    _rxcode, _: 30, 24;
}

impl InterruptCodeRegister {
    /// Gets the generic interrupt code. Useful for determining which interrupt
    /// was raised when the nINT pin is triggered.
    pub fn generic_code(&self) -> InterruptFlagCode {
        self._icode().into()
    }

    /// Gets the number of the filter that matched the recieved message. It's
    /// not clear what the value of this field is when a non-RX interrupt is
    /// raised.
    pub fn filter_hit(&self) -> FilterNumber {
        self._filhit().try_into().unwrap()
    }

    pub fn tx_code(&self) -> TxInterruptFlagCode {
        self._txcode().into()
    }

    pub fn rx_code(&self) -> RxInterruptFlagCode {
        self._rxcode().into()
    }
}

impl_to_from_u32!(InterruptCodeRegister);
impl_register!(InterruptCodeRegister, C1VEC);

bitfield! {
    pub struct InterruptRegister(u32);
    impl Debug;
    u8;
    pub txif, _: 0;
    pub rxif, _: 1;
    _tbcif, _set_tbcif: 2;
    _modif, _set_modif: 3;
    pub tefif, _: 4;
    pub eccif, _: 8;
    pub spicrcif, _: 9;
    pub txatif, _: 10;
    pub rxovif, _: 11;
    _serrif, _set_serrif: 12;
    _cerrif, _set_cerrif: 13;
    _wakif, _set_wakif: 14;
    _ivmif, _set_ivmif: 15;

    pub txie, set_txie: 16;
    pub rxie, set_rxie: 17;
    pub tbcie, set_tbcie: 18;
    pub modie, set_modie: 19;
    pub tefie, set_tefie: 20;
    pub eccie, set_eccie: 24;
    pub spicrcie, set_spicrcie: 25;
    pub txatie, set_txatie: 26;
    pub rxovie, set_rxovie: 27;
    pub serrie, set_serrie: 28;
    pub cerrie, set_cerrie: 29;
    pub wakie, set_wakie: 30;
    pub ivmie, set_ivmie: 31;
}

impl InterruptRegister {
    software_clearable!(tbcif, clear_tbcif);
    software_clearable!(modif, clear_modif);
    software_clearable!(serrif, clear_serrif);
    software_clearable!(cerrif, clear_cerrif);
    software_clearable!(wakif, clear_wakif);
    software_clearable!(ivmif, clear_ivmif);
}

impl_to_from_u32!(InterruptRegister);
impl_register!(InterruptRegister, C1INT);

bitfield! {
    pub struct RxInterruptStatusRegister(u32);
    impl Debug;
    u8;
    pub rfif1, _: 1;
    pub rfif2, _: 2;
    pub rfif3, _: 3;
    pub rfif4, _: 4;
    pub rfif5, _: 5;
    pub rfif6, _: 6;
    pub rfif7, _: 7;
    pub rfif8, _: 8;
    pub rfif9, _: 9;
    pub rfif10, _: 10;
    pub rfif11, _: 11;
    pub rfif12, _: 12;
    pub rfif13, _: 13;
    pub rfif14, _: 14;
    pub rfif15, _: 15;
    pub rfif16, _: 16;
    pub rfif17, _: 17;
    pub rfif18, _: 18;
    pub rfif19, _: 19;
    pub rfif20, _: 20;
    pub rfif21, _: 21;
    pub rfif22, _: 22;
    pub rfif23, _: 23;
    pub rfif24, _: 24;
    pub rfif25, _: 25;
    pub rfif26, _: 26;
    pub rfif27, _: 27;
    pub rfif28, _: 28;
    pub rfif29, _: 29;
    pub rfif30, _: 30;
    pub rfif31, _: 31;
}

impl RxInterruptStatusRegister {
    pub fn get_interrupt(&self, fifo_number: FifoNumber) -> bool {
        self.bit(fifo_number as usize)
    }
}

impl_to_from_u32!(RxInterruptStatusRegister);
impl_register!(RxInterruptStatusRegister, C1RXIF);

bitfield! {
    pub struct RxOverflowInterruptStatusRegister(u32);
    impl Debug;
    u8;
    pub rfovif1, _: 1;
    pub rfovif2, _: 2;
    pub rfovif3, _: 3;
    pub rfovif4, _: 4;
    pub rfovif5, _: 5;
    pub rfovif6, _: 6;
    pub rfovif7, _: 7;
    pub rfovif8, _: 8;
    pub rfovif9, _: 9;
    pub rfovif10, _: 10;
    pub rfovif11, _: 11;
    pub rfovif12, _: 12;
    pub rfovif13, _: 13;
    pub rfovif14, _: 14;
    pub rfovif15, _: 15;
    pub rfovif16, _: 16;
    pub rfovif17, _: 17;
    pub rfovif18, _: 18;
    pub rfovif19, _: 19;
    pub rfovif20, _: 20;
    pub rfovif21, _: 21;
    pub rfovif22, _: 22;
    pub rfovif23, _: 23;
    pub rfovif24, _: 24;
    pub rfovif25, _: 25;
    pub rfovif26, _: 26;
    pub rfovif27, _: 27;
    pub rfovif28, _: 28;
    pub rfovif29, _: 29;
    pub rfovif30, _: 30;
    pub rfovif31, _: 31;
}

impl RxOverflowInterruptStatusRegister {
    pub fn get_interrupt(&self, fifo_number: FifoNumber) -> bool {
        self.bit(fifo_number as usize)
    }
}

impl_to_from_u32!(RxOverflowInterruptStatusRegister);
impl_register!(RxOverflowInterruptStatusRegister, C1RXOVIF);

bitfield! {
    pub struct TxInterruptStatusRegister(u32);
    impl Debug;
    u8;
    pub tfif_txq, _: 0;
    pub tfif1, _: 1;
    pub tfif2, _: 2;
    pub tfif3, _: 3;
    pub tfif4, _: 4;
    pub tfif5, _: 5;
    pub tfif6, _: 6;
    pub tfif7, _: 7;
    pub tfif8, _: 8;
    pub tfif9, _: 9;
    pub tfif10, _: 10;
    pub tfif11, _: 11;
    pub tfif12, _: 12;
    pub tfif13, _: 13;
    pub tfif14, _: 14;
    pub tfif15, _: 15;
    pub tfif16, _: 16;
    pub tfif17, _: 17;
    pub tfif18, _: 18;
    pub tfif19, _: 19;
    pub tfif20, _: 20;
    pub tfif21, _: 21;
    pub tfif22, _: 22;
    pub tfif23, _: 23;
    pub tfif24, _: 24;
    pub tfif25, _: 25;
    pub tfif26, _: 26;
    pub tfif27, _: 27;
    pub tfif28, _: 28;
    pub tfif29, _: 29;
    pub tfif30, _: 30;
    pub tfif31, _: 31;
}

impl TxInterruptStatusRegister {
    pub fn get_tx_queue_interrupt(&self) -> bool {
        self.tfif_txq()
    }

    pub fn get_tx_fifo_interrupt(&self, fifo_number: FifoNumber) -> bool {
        self.bit(fifo_number as usize)
    }
}

impl_to_from_u32!(TxInterruptStatusRegister);
impl_register!(TxInterruptStatusRegister, C1TXIF);

bitfield! {
    pub struct TxAttemptInterruptStatusRegister(u32);
    impl Debug;
    u8;
    pub tfatif_txq, _: 0;
    pub tfatif1, _: 1;
    pub tfatif2, _: 2;
    pub tfatif3, _: 3;
    pub tfatif4, _: 4;
    pub tfatif5, _: 5;
    pub tfatif6, _: 6;
    pub tfatif7, _: 7;
    pub tfatif8, _: 8;
    pub tfatif9, _: 9;
    pub tfatif10, _: 10;
    pub tfatif11, _: 11;
    pub tfatif12, _: 12;
    pub tfatif13, _: 13;
    pub tfatif14, _: 14;
    pub tfatif15, _: 15;
    pub tfatif16, _: 16;
    pub tfatif17, _: 17;
    pub tfatif18, _: 18;
    pub tfatif19, _: 19;
    pub tfatif20, _: 20;
    pub tfatif21, _: 21;
    pub tfatif22, _: 22;
    pub tfatif23, _: 23;
    pub tfatif24, _: 24;
    pub tfatif25, _: 25;
    pub tfatif26, _: 26;
    pub tfatif27, _: 27;
    pub tfatif28, _: 28;
    pub tfatif29, _: 29;
    pub tfatif30, _: 30;
    pub tfatif31, _: 31;
}

impl TxAttemptInterruptStatusRegister {
    pub fn get_tx_queue_interrupt(&self) -> bool {
        self.tfatif_txq()
    }

    pub fn get_tx_fifo_interrupt(&self, fifo_number: FifoNumber) -> bool {
        self.bit(fifo_number as usize)
    }
}

impl_to_from_u32!(TxAttemptInterruptStatusRegister);
impl_register!(TxAttemptInterruptStatusRegister, C1TXATIF);
