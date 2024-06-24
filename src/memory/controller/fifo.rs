use bitfield::bitfield;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::memory::{RepeatedRegister, SFRAddress, RAM_BASE_ADDRESS};
use crate::{impl_register, impl_to_from_u32, software_clearable, software_settable};

pub const HIGHEST_FIFO_PRIORITY: u8 = 0b0001_1111;
pub const LOWEST_FIFO_PRIORITY: u8 = 0;

#[derive(Debug, Clone, Copy, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum FifoNumber {
    Fifo1 = 1,
    Fifo2 = 2,
    Fifo3 = 3,
    Fifo4 = 4,
    Fifo5 = 5,
    Fifo6 = 6,
    Fifo7 = 7,
    Fifo8 = 8,
    Fifo9 = 9,
    Fifo10 = 10,
    Fifo11 = 11,
    Fifo12 = 12,
    Fifo13 = 13,
    Fifo14 = 14,
    Fifo15 = 15,
    Fifo16 = 16,
    Fifo17 = 17,
    Fifo18 = 18,
    Fifo19 = 19,
    Fifo20 = 20,
    Fifo21 = 21,
    Fifo22 = 22,
    Fifo23 = 23,
    Fifo24 = 24,
    Fifo25 = 25,
    Fifo26 = 26,
    Fifo27 = 27,
    Fifo28 = 28,
    Fifo29 = 29,
    Fifo30 = 30,
    Fifo31 = 31,
}

bitfield! {
    pub struct UserAddressRegister(u32);
    u32;
    pub fifoua, _: 31, 0;
}

impl_to_from_u32!(UserAddressRegister);

impl UserAddressRegister {
    pub fn calculate_ram_address(&self) -> u32 {
        self.fifoua() + RAM_BASE_ADDRESS
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum UserAddressKind {
    TxEventFifo,
    TxQueue,
    Fifo(FifoNumber),
}

impl RepeatedRegister for UserAddressRegister {
    type Index = UserAddressKind;

    fn get_address_for(index: Self::Index) -> SFRAddress {
        match index {
            UserAddressKind::TxEventFifo => SFRAddress::C1TEFUA,
            UserAddressKind::TxQueue => SFRAddress::C1TXQUA,
            UserAddressKind::Fifo(fifo_number) => match fifo_number {
                FifoNumber::Fifo1 => SFRAddress::C1FIFOUA1,
                FifoNumber::Fifo2 => SFRAddress::C1FIFOUA2,
                FifoNumber::Fifo3 => SFRAddress::C1FIFOUA3,
                FifoNumber::Fifo4 => SFRAddress::C1FIFOUA4,
                FifoNumber::Fifo5 => SFRAddress::C1FIFOUA5,
                FifoNumber::Fifo6 => SFRAddress::C1FIFOUA6,
                FifoNumber::Fifo7 => SFRAddress::C1FIFOUA7,
                FifoNumber::Fifo8 => SFRAddress::C1FIFOUA8,
                FifoNumber::Fifo9 => SFRAddress::C1FIFOUA9,
                FifoNumber::Fifo10 => SFRAddress::C1FIFOUA10,
                FifoNumber::Fifo11 => SFRAddress::C1FIFOUA11,
                FifoNumber::Fifo12 => SFRAddress::C1FIFOUA12,
                FifoNumber::Fifo13 => SFRAddress::C1FIFOUA13,
                FifoNumber::Fifo14 => SFRAddress::C1FIFOUA14,
                FifoNumber::Fifo15 => SFRAddress::C1FIFOUA15,
                FifoNumber::Fifo16 => SFRAddress::C1FIFOUA16,
                FifoNumber::Fifo17 => SFRAddress::C1FIFOUA17,
                FifoNumber::Fifo18 => SFRAddress::C1FIFOUA18,
                FifoNumber::Fifo19 => SFRAddress::C1FIFOUA19,
                FifoNumber::Fifo20 => SFRAddress::C1FIFOUA20,
                FifoNumber::Fifo21 => SFRAddress::C1FIFOUA21,
                FifoNumber::Fifo22 => SFRAddress::C1FIFOUA22,
                FifoNumber::Fifo23 => SFRAddress::C1FIFOUA23,
                FifoNumber::Fifo24 => SFRAddress::C1FIFOUA24,
                FifoNumber::Fifo25 => SFRAddress::C1FIFOUA25,
                FifoNumber::Fifo26 => SFRAddress::C1FIFOUA26,
                FifoNumber::Fifo27 => SFRAddress::C1FIFOUA27,
                FifoNumber::Fifo28 => SFRAddress::C1FIFOUA28,
                FifoNumber::Fifo29 => SFRAddress::C1FIFOUA29,
                FifoNumber::Fifo30 => SFRAddress::C1FIFOUA30,
                FifoNumber::Fifo31 => SFRAddress::C1FIFOUA31,
            },
        }
    }
}

bitfield! {
    pub struct TxEventFifoControlRegister(u32);
    impl Debug;
    u8;
    pub tefneie, set_tefneie: 0;
    pub tefhie, set_tefhie: 1;
    pub teffie, set_teffie: 2;
    pub tefovie, set_tefovie: 3;
    pub teftsen, set_teftsen: 5;

    _uinc, _set_uinc: 8;
    _freset, _set_freset: 10;

    _fsize, _set_fsize: 28, 24;
}

impl TxEventFifoControlRegister {
    software_settable!(uinc, set_uinc);
    software_settable!(freset, set_freset);

    // Max size is 32
    pub fn fifo_size(&self) -> u8 {
        self._fsize() + 1
    }

    /// Max size is 32.
    pub fn set_fifo_size(&mut self, size: u8) {
        self._set_fsize(match size.cmp(&32u8) {
            core::cmp::Ordering::Greater => 31,
            _ => size - 1,
        });
    }
}

impl_register!(TxEventFifoControlRegister, C1TEFCON);
impl_to_from_u32!(TxEventFifoControlRegister);

bitfield! {
    pub struct TxEventFifoStatusRegister(u32);
    impl Debug;
    u8;
    pub tefneif, _: 0;
    pub tefhif, _: 1;
    pub teffif, _: 2;
    _tefovif, _set_tefovif: 3;
}

impl_register!(TxEventFifoStatusRegister, C1TEFSTA);
impl_to_from_u32!(TxEventFifoStatusRegister);

impl TxEventFifoStatusRegister {
    software_clearable!(tefovif, clear_tefovif);
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum RetransmissionAttempts {
    Disabled = 0,
    ThreeRetries = 1,
    #[default]
    #[num_enum(alternatives = [3])]
    UnlimitedRetries = 2,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum PayloadSize {
    Bytes8 = 0,
    Bytes12 = 1,
    Bytes16 = 2,
    Bytes20 = 3,
    Bytes24 = 4,
    Bytes32 = 5,
    Bytes48 = 6,
    Bytes64 = 7,
}

impl PayloadSize {
    pub fn num_bytes(&self) -> usize {
        match self {
            Self::Bytes8 => 8,
            Self::Bytes12 => 12,
            Self::Bytes16 => 16,
            Self::Bytes20 => 20,
            Self::Bytes24 => 24,
            Self::Bytes32 => 32,
            Self::Bytes48 => 48,
            Self::Bytes64 => 65,
        }
    }
}

bitfield! {
    pub struct TxQueueControlRegister(u32);
    impl Debug;
    u8;
    pub txqnie, set_txqnie: 0;
    pub txqeie, set_txqeie: 2;
    pub txatie, set_txatie: 4;
    pub txen, _: 7;
    _uinc, _set_uinc: 8;
    pub txreq, set_txreq: 9;
    _freset, _set_freset: 10;
    pub txpri, set_txpri: 20, 16;
    _txat, _set_txat: 22, 21;
    _fsize, _set_fsize: 28, 24;
    _plsize, _set_plsize: 31, 29;
}

impl TxQueueControlRegister {
    software_settable!(uinc, set_uinc);
    software_settable!(freset, set_freset);

    pub fn retransmission_attempts(&self) -> RetransmissionAttempts {
        match RetransmissionAttempts::try_from(self._txat()) {
            Ok(val) => val,
            _ => RetransmissionAttempts::UnlimitedRetries,
        }
    }

    pub fn set_retransmission_attempts(&mut self, value: RetransmissionAttempts) {
        self._set_txat(value.into())
    }

    // Max size is 32
    pub fn fifo_size(&self) -> u8 {
        self._fsize() + 1
    }

    /// Max size is 32.
    pub fn set_fifo_size(&mut self, size: u8) {
        self._set_fsize(match size.cmp(&32u8) {
            core::cmp::Ordering::Greater => 31,
            _ => size - 1,
        });
    }

    pub fn payload_size(&self) -> PayloadSize {
        match PayloadSize::try_from(self._plsize()) {
            Ok(val) => val,
            _ => PayloadSize::Bytes8,
        }
    }

    pub fn set_payload_size(&mut self, size: PayloadSize) {
        self._set_plsize(size.into());
    }
}

impl_register!(TxQueueControlRegister, C1TXQCON);
impl_to_from_u32!(TxQueueControlRegister);

bitfield! {
    pub struct TxQueueStatusRegister(u32);
    impl Debug;
    u8;
    pub txqnif, _: 0;
    pub txqeif, _: 2;
    _txatif, _set_txatif: 4;
    _txerr, _set_txerr: 5;
    _txlarb, _set_txlarb: 6;
    _txabt, _set_txabt: 7;
    pub txqci, _: 12, 8;
}

impl_register!(TxQueueStatusRegister, C1TXQSTA);
impl_to_from_u32!(TxQueueStatusRegister);

impl TxQueueStatusRegister {
    software_clearable!(txatif, clear_txatif);
    software_clearable!(txerr, clear_txerr);
    software_clearable!(txlarb, clear_txlarb);
    software_clearable!(txabt, clear_txabt);
}

bitfield! {
    pub struct FifoControlRegister(u32);
    u8;
    pub tfnrfnie, set_tfnrfnie: 0;
    pub tfhrfhie, set_tfhrfhie: 1;
    pub tferffie, set_tferffie: 2;
    pub rxovie, set_rxovie: 3;
    pub txatie, set_txatie: 4;
    pub rxtsen, set_rxtsen: 5;
    pub rtren, set_rtren: 6;
    pub txen, set_txen: 7;
    _uinc, _set_uinc: 8;
    pub txreq, set_txreq: 9;
    _freset, _set_freset: 10;
    pub txpri, set_txpri: 20, 16;
    _txat, _set_txat: 22, 21;
    _fsize, _set_fsize: 28, 24;
    _plsize, _set_plsize: 31, 29;
}

impl_to_from_u32!(FifoControlRegister);

impl FifoControlRegister {
    software_settable!(uinc, set_uinc);
    software_settable!(freset, set_freset);

    pub fn retransmission_attempts(&self) -> RetransmissionAttempts {
        match RetransmissionAttempts::try_from(self._txat()) {
            Ok(val) => val,
            _ => RetransmissionAttempts::UnlimitedRetries,
        }
    }

    pub fn set_retransmission_attempts(&mut self, value: RetransmissionAttempts) {
        self._set_txat(value.into())
    }

    // Max size is 32
    pub fn fifo_size(&self) -> u8 {
        self._fsize() + 1
    }

    /// Max size is 32.
    pub fn set_fifo_size(&mut self, size: u8) {
        self._set_fsize(match size.cmp(&32u8) {
            core::cmp::Ordering::Greater => 31,
            _ => size - 1,
        });
    }

    pub fn payload_size(&self) -> PayloadSize {
        match PayloadSize::try_from(self._plsize()) {
            Ok(val) => val,
            _ => PayloadSize::Bytes8,
        }
    }

    pub fn set_payload_size(&mut self, size: PayloadSize) {
        self._set_plsize(size.into());
    }
}

impl RepeatedRegister for FifoControlRegister {
    type Index = FifoNumber;

    fn get_address_for(fifo_number: Self::Index) -> SFRAddress {
        match fifo_number {
            FifoNumber::Fifo1 => SFRAddress::C1FIFOCON1,
            FifoNumber::Fifo2 => SFRAddress::C1FIFOCON2,
            FifoNumber::Fifo3 => SFRAddress::C1FIFOCON3,
            FifoNumber::Fifo4 => SFRAddress::C1FIFOCON4,
            FifoNumber::Fifo5 => SFRAddress::C1FIFOCON5,
            FifoNumber::Fifo6 => SFRAddress::C1FIFOCON6,
            FifoNumber::Fifo7 => SFRAddress::C1FIFOCON7,
            FifoNumber::Fifo8 => SFRAddress::C1FIFOCON8,
            FifoNumber::Fifo9 => SFRAddress::C1FIFOCON9,
            FifoNumber::Fifo10 => SFRAddress::C1FIFOCON10,
            FifoNumber::Fifo11 => SFRAddress::C1FIFOCON11,
            FifoNumber::Fifo12 => SFRAddress::C1FIFOCON12,
            FifoNumber::Fifo13 => SFRAddress::C1FIFOCON13,
            FifoNumber::Fifo14 => SFRAddress::C1FIFOCON14,
            FifoNumber::Fifo15 => SFRAddress::C1FIFOCON15,
            FifoNumber::Fifo16 => SFRAddress::C1FIFOCON16,
            FifoNumber::Fifo17 => SFRAddress::C1FIFOCON17,
            FifoNumber::Fifo18 => SFRAddress::C1FIFOCON18,
            FifoNumber::Fifo19 => SFRAddress::C1FIFOCON19,
            FifoNumber::Fifo20 => SFRAddress::C1FIFOCON20,
            FifoNumber::Fifo21 => SFRAddress::C1FIFOCON21,
            FifoNumber::Fifo22 => SFRAddress::C1FIFOCON22,
            FifoNumber::Fifo23 => SFRAddress::C1FIFOCON23,
            FifoNumber::Fifo24 => SFRAddress::C1FIFOCON24,
            FifoNumber::Fifo25 => SFRAddress::C1FIFOCON25,
            FifoNumber::Fifo26 => SFRAddress::C1FIFOCON26,
            FifoNumber::Fifo27 => SFRAddress::C1FIFOCON27,
            FifoNumber::Fifo28 => SFRAddress::C1FIFOCON28,
            FifoNumber::Fifo29 => SFRAddress::C1FIFOCON29,
            FifoNumber::Fifo30 => SFRAddress::C1FIFOCON30,
            FifoNumber::Fifo31 => SFRAddress::C1FIFOCON31,
        }
    }
}

bitfield! {
    pub struct FifoStatusRegister(u32);
    u8;
    pub tfnrfnif, _: 0;
    pub tfhrfhif, _: 1;
    pub tferffif, _: 2;
    _rxovif, _set_rxovif: 3;
    _txatif, _set_txatif: 4;
    _txerr, _set_txerr: 5;
    _txlarb, _set_txlarb: 6;
    _txabt, _set_txabt: 7;
    pub fifoci, _: 12, 8;
}

impl_to_from_u32!(FifoStatusRegister);

impl FifoStatusRegister {
    software_clearable!(rxovif, clear_rxovif);
    software_clearable!(txatif, clear_txatif);
    software_clearable!(txerr, clear_txerr);
    software_clearable!(txlarb, clear_txlarb);
    software_clearable!(txabt, clear_txabt);
}

impl RepeatedRegister for FifoStatusRegister {
    type Index = FifoNumber;

    fn get_address_for(fifo_number: Self::Index) -> SFRAddress {
        match fifo_number {
            FifoNumber::Fifo1 => SFRAddress::C1FIFOSTA1,
            FifoNumber::Fifo2 => SFRAddress::C1FIFOSTA2,
            FifoNumber::Fifo3 => SFRAddress::C1FIFOSTA3,
            FifoNumber::Fifo4 => SFRAddress::C1FIFOSTA4,
            FifoNumber::Fifo5 => SFRAddress::C1FIFOSTA5,
            FifoNumber::Fifo6 => SFRAddress::C1FIFOSTA6,
            FifoNumber::Fifo7 => SFRAddress::C1FIFOSTA7,
            FifoNumber::Fifo8 => SFRAddress::C1FIFOSTA8,
            FifoNumber::Fifo9 => SFRAddress::C1FIFOSTA9,
            FifoNumber::Fifo10 => SFRAddress::C1FIFOSTA10,
            FifoNumber::Fifo11 => SFRAddress::C1FIFOSTA11,
            FifoNumber::Fifo12 => SFRAddress::C1FIFOSTA12,
            FifoNumber::Fifo13 => SFRAddress::C1FIFOSTA13,
            FifoNumber::Fifo14 => SFRAddress::C1FIFOSTA14,
            FifoNumber::Fifo15 => SFRAddress::C1FIFOSTA15,
            FifoNumber::Fifo16 => SFRAddress::C1FIFOSTA16,
            FifoNumber::Fifo17 => SFRAddress::C1FIFOSTA17,
            FifoNumber::Fifo18 => SFRAddress::C1FIFOSTA18,
            FifoNumber::Fifo19 => SFRAddress::C1FIFOSTA19,
            FifoNumber::Fifo20 => SFRAddress::C1FIFOSTA20,
            FifoNumber::Fifo21 => SFRAddress::C1FIFOSTA21,
            FifoNumber::Fifo22 => SFRAddress::C1FIFOSTA22,
            FifoNumber::Fifo23 => SFRAddress::C1FIFOSTA23,
            FifoNumber::Fifo24 => SFRAddress::C1FIFOSTA24,
            FifoNumber::Fifo25 => SFRAddress::C1FIFOSTA25,
            FifoNumber::Fifo26 => SFRAddress::C1FIFOSTA26,
            FifoNumber::Fifo27 => SFRAddress::C1FIFOSTA27,
            FifoNumber::Fifo28 => SFRAddress::C1FIFOSTA28,
            FifoNumber::Fifo29 => SFRAddress::C1FIFOSTA29,
            FifoNumber::Fifo30 => SFRAddress::C1FIFOSTA30,
            FifoNumber::Fifo31 => SFRAddress::C1FIFOSTA31,
        }
    }
}
