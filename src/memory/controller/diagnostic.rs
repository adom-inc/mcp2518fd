use bitfield::bitfield;

use crate::{impl_register, impl_to_from_u32};

bitfield! {
    pub struct TransmitReceiveErrorCountRegister(u32);
    impl Debug;
    u8;
    pub tec, _: 7, 0;
    pub rec, _: 15, 8;
    pub ewarn, _: 16;
    pub rxwarn, _: 17;
    pub txwarn, _: 18;
    pub rxbp, _: 19;
    pub txbp, _: 20;
    pub txbo, _: 21;
}

impl_to_from_u32!(TransmitReceiveErrorCountRegister);
impl_register!(TransmitReceiveErrorCountRegister, C1TREC);

bitfield! {
    pub struct BusDiagnosticRegister0(u32);
    impl Debug;
    u8;
    pub nrerrcnt, set_nrerrcnt: 7, 0;
    pub nterrcnt, set_nterrcnt: 15, 8;
    pub drerrcnt, set_drerrcnt: 23, 16;
    pub dterrcnt, set_dterrcnt: 31, 24;
}

impl_to_from_u32!(BusDiagnosticRegister0);
impl_register!(BusDiagnosticRegister0, C1BDIAG0);

bitfield! {
    pub struct BusDiagnosticRegister1(u32);
    impl Debug;
    u8;
    pub u16, efmsgcnt, set_efmsgcnt: 15, 0;
    pub nbit0err, set_nbit0err: 16;
    pub nbit1err, set_nbit1err: 17;
    pub nackerr, set_nackerr: 18;
    pub nformerr, set_nformerr: 19;
    pub nstuferr, set_nstuferr: 20;
    pub ncrcerr, set_ncrcerr: 21;
    pub txboerr, set_txboerr: 23;
    pub dbit0err, set_dbit0err: 24;
    pub dbit1err, set_dbit1err: 25;
    pub dformerr, set_dformerr: 27;
    pub dstuferr, set_dstuferr: 28;
    pub dcrcerr, set_dcrcerr: 29;
    pub esi, set_esi: 30;
    pub dlcmm, set_dlcmm: 31;
}

impl_to_from_u32!(BusDiagnosticRegister1);
impl_register!(BusDiagnosticRegister1, C1BDIAG1);
