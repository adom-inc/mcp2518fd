use bitfield::bitfield;

use crate::{impl_register, impl_to_from_u32};

bitfield! {
    pub struct OscillatorControlRegister(u32);
    impl Debug;
    u8;
    pub pllen, set_pllen: 0;
    pub oscdis, set_oscdis: 2;
    pub slckdiv, set_slckdiv: 4;
    pub clkodiv, set_clkodiv: 6, 5;
    pub pllrdy, _: 8;
    pub oscrdy, _: 10;
    pub sclkrdy, _: 12;
}

impl_to_from_u32!(OscillatorControlRegister);
impl_register!(OscillatorControlRegister, OSC);

bitfield! {
    pub struct IoControlRegister(u32);
    impl Debug;
    u8;
    pub tris0, set_tris0: 0;
    pub tris1, set_tris1: 1;
    pub xstbyen, set_xstbyen: 6;
    pub lat0, set_lat0: 8;
    pub lat1, set_lat1: 9;
    pub gpio0, _: 16;
    pub gpio1, _: 17;
    pub pm0, set_pm0: 24;
    pub pm1, set_pm1: 25;
    pub txcanod, set_txcanod: 28;
    pub sof, set_sof: 29;
    pub intod, set_intod: 30;
}

impl_to_from_u32!(IoControlRegister);
impl_register!(IoControlRegister, IOCON);

bitfield! {
    pub struct CrcRegister(u32);
    impl Debug;
    u8;
    pub u16, crc, _: 0, 15;
    _crcerrif, _set_crcerrif: 16;
    _ferrif, _set_ferrif: 17;
    pub crcerrie, set_crcerrie: 24;
    pub ferrie, set_ferrie: 25;
}

impl CrcRegister {
    pub fn crcerrif(&self) -> bool {
        self._crcerrif()
    }

    pub fn clear_crcerrif(&mut self) {
        self._set_crcerrif(false)
    }

    pub fn ferrif(&self) -> bool {
        self._ferrif()
    }

    pub fn clear_ferrif(&mut self) {
        self._set_ferrif(false)
    }
}

impl_to_from_u32!(CrcRegister);
impl_register!(CrcRegister, CRC);

bitfield! {
    pub struct EccControlRegister(u32);
    impl Debug;
    u8;
    pub eccen, set_eccen: 0;
    pub secie, set_secie: 1;
    pub dedie, set_dedie: 2;
    pub parity, set_parity: 14, 8;
}

impl_to_from_u32!(EccControlRegister);
impl_register!(EccControlRegister, ECCCON);

bitfield! {
    pub struct EccStatusRegister(u32);
    impl Debug;
    u8;
    _secif, _set_secif: 1;
    _dedif, _set_dedif: 2;
    pub u16, erraddr, _: 27, 16;
}

impl EccStatusRegister {
    pub fn secif(&self) -> bool {
        self._secif()
    }

    pub fn clear_secif(&mut self) {
        self._set_secif(false)
    }

    pub fn dedif(&self) -> bool {
        self._dedif()
    }

    pub fn clear_dedif(&mut self) {
        self._set_dedif(false)
    }
}

impl_to_from_u32!(EccStatusRegister);
impl_register!(EccStatusRegister, ECCSTAT);

bitfield! {
    pub struct DeviceIdRegister(u32);
    impl Debug;
    u8;
    pub rev, _: 3, 0;
    pub id, _: 7, 4;
}

impl_to_from_u32!(DeviceIdRegister);
impl_register!(DeviceIdRegister, DEVID);
