use bitfield::bitfield;

use crate::{impl_register, impl_to_from_u32, software_settable};

bitfield! {
    pub struct TransmitRequestRegister(u32);
    impl Debug;
    u8;
    _txreq_txq, _set_txreq_txq: 0;
    _txreq1, _set_txreq1: 1;
    _txreq2, _set_txreq2: 2;
    _txreq3, _set_txreq3: 3;
    _txreq4, _set_txreq4: 4;
    _txreq5, _set_txreq5: 5;
    _txreq6, _set_txreq6: 6;
    _txreq7, _set_txreq7: 7;
    _txreq8, _set_txreq8: 8;
    _txreq9, _set_txreq9: 9;
    _txreq10, _set_txreq10: 10;
    _txreq11, _set_txreq11: 11;
    _txreq12, _set_txreq12: 12;
    _txreq13, _set_txreq13: 13;
    _txreq14, _set_txreq14: 14;
    _txreq15, _set_txreq15: 15;
    _txreq16, _set_txreq16: 16;
    _txreq17, _set_txreq17: 17;
    _txreq18, _set_txreq18: 18;
    _txreq19, _set_txreq19: 19;
    _txreq20, _set_txreq20: 20;
    _txreq21, _set_txreq21: 21;
    _txreq22, _set_txreq22: 22;
    _txreq23, _set_txreq23: 23;
    _txreq24, _set_txreq24: 24;
    _txreq25, _set_txreq25: 25;
    _txreq26, _set_txreq26: 26;
    _txreq27, _set_txreq27: 27;
    _txreq28, _set_txreq28: 28;
    _txreq29, _set_txreq29: 29;
    _txreq30, _set_txreq30: 30;
    _txreq31, _set_txreq31: 31;
}

impl TransmitRequestRegister {
    software_settable!(txreq_txq, set_txreq_txq);
    software_settable!(txreq1, set_txreq1);
    software_settable!(txreq2, set_txreq2);
    software_settable!(txreq3, set_txreq3);
    software_settable!(txreq4, set_txreq4);
    software_settable!(txreq5, set_txreq5);
    software_settable!(txreq6, set_txreq6);
    software_settable!(txreq7, set_txreq7);
    software_settable!(txreq8, set_txreq8);
    software_settable!(txreq9, set_txreq9);
    software_settable!(txreq10, set_txreq10);
    software_settable!(txreq11, set_txreq11);
    software_settable!(txreq12, set_txreq12);
    software_settable!(txreq13, set_txreq13);
    software_settable!(txreq14, set_txreq14);
    software_settable!(txreq15, set_txreq15);
    software_settable!(txreq16, set_txreq16);
    software_settable!(txreq17, set_txreq17);
    software_settable!(txreq18, set_txreq18);
    software_settable!(txreq19, set_txreq19);
    software_settable!(txreq20, set_txreq20);
    software_settable!(txreq21, set_txreq21);
    software_settable!(txreq22, set_txreq22);
    software_settable!(txreq23, set_txreq23);
    software_settable!(txreq24, set_txreq24);
    software_settable!(txreq25, set_txreq25);
    software_settable!(txreq26, set_txreq26);
    software_settable!(txreq27, set_txreq27);
    software_settable!(txreq28, set_txreq28);
    software_settable!(txreq29, set_txreq29);
    software_settable!(txreq30, set_txreq30);
    software_settable!(txreq31, set_txreq31);
}

impl_to_from_u32!(TransmitRequestRegister);
impl_register!(TransmitRequestRegister, C1TXREQ);
