use bitfield::bitfield;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::impl_to_from_u32;
use crate::memory::{RepeatedRegister, SFRAddress};

use super::fifo::FifoNumber;

#[derive(Clone, Copy, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum FilterControlNumber {
    FilterControl0 = 0,
    FilterControl1 = 1,
    FilterControl2 = 2,
    FilterControl3 = 3,
    FilterControl4 = 4,
    FilterControl5 = 5,
    FilterControl6 = 6,
    FilterControl7 = 7,
}

#[derive(Clone, Copy, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum FilterControlIndex {
    Filter0 = 0,
    Filter1 = 1,
    Filter2 = 2,
    Filter3 = 3,
}

#[derive(Clone, Copy, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum FilterNumber {
    Filter0 = 0,
    Filter1 = 1,
    Filter2 = 2,
    Filter3 = 3,
    Filter4 = 4,
    Filter5 = 5,
    Filter6 = 6,
    Filter7 = 7,
    Filter8 = 8,
    Filter9 = 9,
    Filter10 = 10,
    Filter11 = 11,
    Filter12 = 12,
    Filter13 = 13,
    Filter14 = 14,
    Filter15 = 15,
    Filter16 = 16,
    Filter17 = 17,
    Filter18 = 18,
    Filter19 = 19,
    Filter20 = 20,
    Filter21 = 21,
    Filter22 = 22,
    Filter23 = 23,
    Filter24 = 24,
    Filter25 = 25,
    Filter26 = 26,
    Filter27 = 27,
    Filter28 = 28,
    Filter29 = 29,
    Filter30 = 30,
    Filter31 = 31,
}

impl FilterNumber {
    pub fn get_control_register(&self) -> (FilterControlNumber, FilterControlIndex) {
        let control_number = *self as u8 / 4;
        let index = *self as u8 % 4;

        (
            control_number.try_into().unwrap(),
            index.try_into().unwrap(),
        )
    }
}

bitfield! {
    pub struct FilterControlRegister(u32);
    impl Debug;
    u8;
    pub f0bp, set_f0bp: 4, 0;
    pub flten0, set_flten0: 7;
    pub f1bp, set_f1bp: 12, 8;
    pub flten1, set_flten1: 15;
    pub f2bp, set_f2bp: 20, 16;
    pub flten2, set_flten2: 23;
    pub f3bp, set_f3bp: 28, 24;
    pub flten3, set_flten3: 31;
}

impl_to_from_u32!(FilterControlRegister);

impl RepeatedRegister for FilterControlRegister {
    type Index = FilterControlNumber;

    fn get_address_for(index: Self::Index) -> SFRAddress {
        match index {
            FilterControlNumber::FilterControl0 => SFRAddress::C1FLTCON0,
            FilterControlNumber::FilterControl1 => SFRAddress::C1FLTCON1,
            FilterControlNumber::FilterControl2 => SFRAddress::C1FLTCON2,
            FilterControlNumber::FilterControl3 => SFRAddress::C1FLTCON3,
            FilterControlNumber::FilterControl4 => SFRAddress::C1FLTCON4,
            FilterControlNumber::FilterControl5 => SFRAddress::C1FLTCON5,
            FilterControlNumber::FilterControl6 => SFRAddress::C1FLTCON6,
            FilterControlNumber::FilterControl7 => SFRAddress::C1FLTCON7,
        }
    }
}

impl FilterControlRegister {
    pub fn is_enabled(&self, index: FilterControlIndex) -> bool {
        match index {
            FilterControlIndex::Filter0 => self.flten0(),
            FilterControlIndex::Filter1 => self.flten1(),
            FilterControlIndex::Filter2 => self.flten2(),
            FilterControlIndex::Filter3 => self.flten3(),
        }
    }

    pub fn set_enabled(&mut self, index: FilterControlIndex, enabled: bool) {
        match index {
            FilterControlIndex::Filter0 => self.set_flten0(enabled),
            FilterControlIndex::Filter1 => self.set_flten1(enabled),
            FilterControlIndex::Filter2 => self.set_flten2(enabled),
            FilterControlIndex::Filter3 => self.set_flten3(enabled),
        }
    }

    pub fn get_buffer_pointer(&self, index: FilterControlIndex) -> Option<FifoNumber> {
        match index {
            FilterControlIndex::Filter0 => self.f0bp().try_into().ok(),
            FilterControlIndex::Filter1 => self.f1bp().try_into().ok(),
            FilterControlIndex::Filter2 => self.f2bp().try_into().ok(),
            FilterControlIndex::Filter3 => self.f3bp().try_into().ok(),
        }
    }

    pub fn set_buffer_pointer(&mut self, index: FilterControlIndex, fifo_number: FifoNumber) {
        match index {
            FilterControlIndex::Filter0 => self.set_f0bp(fifo_number.into()),
            FilterControlIndex::Filter1 => self.set_f1bp(fifo_number.into()),
            FilterControlIndex::Filter2 => self.set_f2bp(fifo_number.into()),
            FilterControlIndex::Filter3 => self.set_f3bp(fifo_number.into()),
        }
    }
}

bitfield! {
    pub struct FilterObjectRegister(u32);
    impl Debug;
    u8;
    pub u16, sid, set_sid: 10, 0;
    pub u32, eid, set_eid: 28, 11;
    pub sid11, set_sid11: 29;
    pub exide, set_exide: 30;
}

impl_to_from_u32!(FilterObjectRegister);

impl RepeatedRegister for FilterObjectRegister {
    type Index = FilterNumber;

    fn get_address_for(index: Self::Index) -> SFRAddress {
        match index {
            FilterNumber::Filter0 => SFRAddress::C1FLTOBJ0,
            FilterNumber::Filter1 => SFRAddress::C1FLTOBJ1,
            FilterNumber::Filter2 => SFRAddress::C1FLTOBJ2,
            FilterNumber::Filter3 => SFRAddress::C1FLTOBJ3,
            FilterNumber::Filter4 => SFRAddress::C1FLTOBJ4,
            FilterNumber::Filter5 => SFRAddress::C1FLTOBJ5,
            FilterNumber::Filter6 => SFRAddress::C1FLTOBJ6,
            FilterNumber::Filter7 => SFRAddress::C1FLTOBJ7,
            FilterNumber::Filter8 => SFRAddress::C1FLTOBJ8,
            FilterNumber::Filter9 => SFRAddress::C1FLTOBJ9,
            FilterNumber::Filter10 => SFRAddress::C1FLTOBJ10,
            FilterNumber::Filter11 => SFRAddress::C1FLTOBJ11,
            FilterNumber::Filter12 => SFRAddress::C1FLTOBJ12,
            FilterNumber::Filter13 => SFRAddress::C1FLTOBJ13,
            FilterNumber::Filter14 => SFRAddress::C1FLTOBJ14,
            FilterNumber::Filter15 => SFRAddress::C1FLTOBJ15,
            FilterNumber::Filter16 => SFRAddress::C1FLTOBJ16,
            FilterNumber::Filter17 => SFRAddress::C1FLTOBJ17,
            FilterNumber::Filter18 => SFRAddress::C1FLTOBJ18,
            FilterNumber::Filter19 => SFRAddress::C1FLTOBJ19,
            FilterNumber::Filter20 => SFRAddress::C1FLTOBJ20,
            FilterNumber::Filter21 => SFRAddress::C1FLTOBJ21,
            FilterNumber::Filter22 => SFRAddress::C1FLTOBJ22,
            FilterNumber::Filter23 => SFRAddress::C1FLTOBJ23,
            FilterNumber::Filter24 => SFRAddress::C1FLTOBJ24,
            FilterNumber::Filter25 => SFRAddress::C1FLTOBJ25,
            FilterNumber::Filter26 => SFRAddress::C1FLTOBJ26,
            FilterNumber::Filter27 => SFRAddress::C1FLTOBJ27,
            FilterNumber::Filter28 => SFRAddress::C1FLTOBJ28,
            FilterNumber::Filter29 => SFRAddress::C1FLTOBJ29,
            FilterNumber::Filter30 => SFRAddress::C1FLTOBJ30,
            FilterNumber::Filter31 => SFRAddress::C1FLTOBJ31,
        }
    }
}

bitfield! {
    pub struct MaskRegister(u32);
    impl Debug;
    u8;
    pub u16, msid, set_msid: 10, 0;
    pub u32, meid, set_meid: 28, 11;
    pub msid11, set_msid11: 29;
    pub mide, set_mide: 30;
}

impl_to_from_u32!(MaskRegister);

impl RepeatedRegister for MaskRegister {
    type Index = FilterNumber;

    fn get_address_for(index: Self::Index) -> SFRAddress {
        match index {
            FilterNumber::Filter0 => SFRAddress::C1MASK0,
            FilterNumber::Filter1 => SFRAddress::C1MASK1,
            FilterNumber::Filter2 => SFRAddress::C1MASK2,
            FilterNumber::Filter3 => SFRAddress::C1MASK3,
            FilterNumber::Filter4 => SFRAddress::C1MASK4,
            FilterNumber::Filter5 => SFRAddress::C1MASK5,
            FilterNumber::Filter6 => SFRAddress::C1MASK6,
            FilterNumber::Filter7 => SFRAddress::C1MASK7,
            FilterNumber::Filter8 => SFRAddress::C1MASK8,
            FilterNumber::Filter9 => SFRAddress::C1MASK9,
            FilterNumber::Filter10 => SFRAddress::C1MASK10,
            FilterNumber::Filter11 => SFRAddress::C1MASK11,
            FilterNumber::Filter12 => SFRAddress::C1MASK12,
            FilterNumber::Filter13 => SFRAddress::C1MASK13,
            FilterNumber::Filter14 => SFRAddress::C1MASK14,
            FilterNumber::Filter15 => SFRAddress::C1MASK15,
            FilterNumber::Filter16 => SFRAddress::C1MASK16,
            FilterNumber::Filter17 => SFRAddress::C1MASK17,
            FilterNumber::Filter18 => SFRAddress::C1MASK18,
            FilterNumber::Filter19 => SFRAddress::C1MASK19,
            FilterNumber::Filter20 => SFRAddress::C1MASK20,
            FilterNumber::Filter21 => SFRAddress::C1MASK21,
            FilterNumber::Filter22 => SFRAddress::C1MASK22,
            FilterNumber::Filter23 => SFRAddress::C1MASK23,
            FilterNumber::Filter24 => SFRAddress::C1MASK24,
            FilterNumber::Filter25 => SFRAddress::C1MASK25,
            FilterNumber::Filter26 => SFRAddress::C1MASK26,
            FilterNumber::Filter27 => SFRAddress::C1MASK27,
            FilterNumber::Filter28 => SFRAddress::C1MASK28,
            FilterNumber::Filter29 => SFRAddress::C1MASK29,
            FilterNumber::Filter30 => SFRAddress::C1MASK30,
            FilterNumber::Filter31 => SFRAddress::C1MASK31,
        }
    }
}
