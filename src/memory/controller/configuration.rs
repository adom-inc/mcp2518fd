use core::convert::TryFrom;

use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};

use bitfield::bitfield;

use crate::{impl_register, impl_to_from_u32};

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum DataBits {
    Bits1 = 1,
    Bits2 = 2,
    Bits3 = 3,
    Bits4 = 4,
    Bits5 = 5,
    Bits6 = 6,
    Bits7 = 7,
    Bits8 = 8,
    Bits9 = 9,
    Bits10 = 10,
    Bits11 = 11,
    Bits12 = 12,
    Bits13 = 13,
    Bits14 = 14,
    Bits15 = 15,
    Bits16 = 16,
    Bits17 = 17,
    Bits18 = 18,
}

#[derive(Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum WakeupFilterTime {
    T00Filter = 0,
    T01Filter = 1,
    T10Filter = 2,
    T11Filter = 3,
}

#[derive(Clone, Copy, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum OperationMode {
    NormalCanFD = 0,
    Sleep = 1,
    InternalLoopback = 2,
    ListenOnly = 3,
    Configuration = 4,
    ExternalLoopback = 5,
    NormalCan2 = 6,
    Restricted = 7,
    Unknown = 0xff,
}

/// All times are in arbitration bit times
#[derive(Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum InterTransmissionDelay {
    NoDelay = 0,
    Delay2 = 1,
    Delay4 = 2,
    Delay8 = 3,
    Delay16 = 4,
    Delay32 = 5,
    Delay64 = 6,
    Delay128 = 7,
    Delay256 = 8,
    Delay512 = 9,
    Delay1024 = 10,
    Delay2048 = 11,
    Delay4096 = 12,
}

bitfield! {
    pub struct CanControlRegister(u32);
    impl Debug;
    u8;
    _dncnt, _set_dncnt: 4, 0;
    pub isocrcen, set_isocrcen: 5;
    pub pxedis, set_pxedis: 6;
    pub wakfil, set_wakfil: 8;
    _wft, _set_wft: 10, 9;
    pub busy, _: 11;
    pub brsdis, set_brsdis: 12;
    pub rtxat, set_rtxat: 16;
    pub esigm, set_esigm: 17;
    pub serr2lom, set_serr2lom: 18;
    pub stef, set_stef: 19;
    pub txqen, set_txqen: 20;
    _opmod, _: 23, 21;
    _, _set_reqop: 26, 24;
    pub abat, set_abat: 27;
    _txbws, _set_txbws: 31, 28;
}

impl_to_from_u32!(CanControlRegister);
impl_register!(CanControlRegister, C1CON);

impl CanControlRegister {
    pub fn dncnt(&self) -> Result<DataBits, TryFromPrimitiveError<DataBits>> {
        DataBits::try_from(self._dncnt())
    }

    pub fn set_dncnt(&mut self, bits: DataBits) {
        self._set_dncnt(bits.into())
    }

    pub fn wft(&self) -> Result<WakeupFilterTime, TryFromPrimitiveError<WakeupFilterTime>> {
        WakeupFilterTime::try_from(self._wft())
    }

    pub fn set_wft(&mut self, filter: WakeupFilterTime) {
        self._set_wft(filter.into())
    }

    pub fn opmode(&self) -> OperationMode {
        match OperationMode::try_from(self._opmod()) {
            Ok(val) => val,
            Err(_) => OperationMode::Unknown,
        }
    }

    pub fn set_opmode(&mut self, mode: OperationMode) {
        self._set_reqop(mode.into());
    }

    pub fn txbws(
        &self,
    ) -> Result<InterTransmissionDelay, TryFromPrimitiveError<InterTransmissionDelay>> {
        InterTransmissionDelay::try_from(self._txbws())
    }

    pub fn set_txbws(&mut self, delay: InterTransmissionDelay) {
        self._set_txbws(delay.into());
    }
}

bitfield! {
    pub struct NominalBitTimeConfigurationRegister(u32);
    impl Debug;
    u8;
    pub sjw, set_sjw: 6, 0;
    pub tseg2, set_tseg2: 14, 8;
    pub tseg1, set_tseg1: 23, 16;
    pub brp, set_brp: 31, 24;
}

impl_to_from_u32!(NominalBitTimeConfigurationRegister);
impl_register!(NominalBitTimeConfigurationRegister, C1NBTCFG);

bitfield! {
    pub struct DataBitTimeConfigurationRegister(u32);
    impl Debug;
    u8;
    pub sjw, set_sjw: 3, 0;
    pub tseg2, set_tseg2: 11, 8;
    pub tseg1, set_tseg1: 20, 16;
    pub brp, set_brp: 31, 24;
}

impl_to_from_u32!(DataBitTimeConfigurationRegister);
impl_register!(DataBitTimeConfigurationRegister, C1DBTCFG);

#[derive(Clone, Copy, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum TransmitterDelayCompensationMode {
    Disabled = 0,
    Manual = 1,
    #[num_enum(alternatives = [3])]
    Automatic = 2,
}

bitfield! {
    pub struct TransmitterDelayCompensationRegister(u32);
    impl Debug;
    u8;
    pub tdcv, set_tdcv: 5, 0;
    pub tdco, set_tdco: 13, 8;
    _tdcmod, _set_tdcmod: 17, 16;
    pub sid11en, set_sid11en: 24;
    pub edgflten, set_edgflten: 25;
}

impl TransmitterDelayCompensationRegister {
    pub fn tdcmod(
        &self,
    ) -> Result<
        TransmitterDelayCompensationMode,
        TryFromPrimitiveError<TransmitterDelayCompensationMode>,
    > {
        TransmitterDelayCompensationMode::try_from(self._tdcmod())
    }

    pub fn set_tdcmod(&mut self, filter: TransmitterDelayCompensationMode) {
        self._set_tdcmod(filter.into())
    }
}

impl_to_from_u32!(TransmitterDelayCompensationRegister);
impl_register!(TransmitterDelayCompensationRegister, C1TDC);

bitfield! {
    pub struct TimeBaseCounterRegister(u32);
    impl Debug;
    u32;
    pub tbc, set_tbc: 31, 0;
}

impl_to_from_u32!(TimeBaseCounterRegister);
impl_register!(TimeBaseCounterRegister, C1TBC);

bitfield! {
    pub struct TimeStampControlRegister(u32);
    impl Debug;
    u8;
    pub u16, tbcpre, set_tbcpre: 9, 0;
    pub tbcen, set_tbcen: 16;
    pub tseof, set_tseof: 17;
    pub tsres, set_tsres: 18;
}

impl_to_from_u32!(TimeStampControlRegister);
impl_register!(TimeStampControlRegister, C1TSCON);
