use bitfield::bitfield;
use embedded_can::{Id, StandardId};

use super::{dlc_for_len, len_for_dlc, HEADER_SIZE_DWORDS, MAX_FD_BUFFER_SIZE};

bitfield! {
    pub struct TxHeader([u32]);
    impl Debug;
    u8;

    // T0
    pub u16, sid, set_sid: 10, 0;
    pub u32, eid, set_eid: 28, 11;
    pub sid11, set_sid11: 29;

    // T1
    pub dlc, set_dlc: 35, 32;
    pub ide, set_ide: 36;
    pub rtr, set_rtr: 37;
    pub brs, set_brs: 38;
    pub fdf, set_fdf: 39;
    pub esi, set_esi: 40;
    pub u32, seq, set_seq: 63, 41;
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TxMessage {
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    header: TxHeader<[u32; HEADER_SIZE_DWORDS]>,
    data: [u8; MAX_FD_BUFFER_SIZE],
    data_len: usize,
}

impl TxMessage {
    pub fn new_fd(identifier: Id, data: &[u8]) -> Option<Self> {
        Self::new(identifier, data, true)
    }

    pub fn new_2_0(identifier: Id, data: &[u8]) -> Option<Self> {
        Self::new(identifier, data, false)
    }

    fn new(identifier: Id, data: &[u8], is_fd: bool) -> Option<Self> {
        let mut header = TxHeader([0u32; HEADER_SIZE_DWORDS]);

        let dlc = dlc_for_len(data.len(), is_fd)?;

        header.set_dlc(dlc);
        header.set_fdf(is_fd);

        match identifier {
            Id::Standard(id) => {
                header.set_sid(id.as_raw());
            }
            Id::Extended(id) => {
                header.set_sid(id.standard_id().as_raw());
                header.set_eid(id.as_raw() & ((!StandardId::MAX.as_raw() as u32) >> 11));
                header.set_ide(true);
            }
        }

        let mut data_buf = [0u8; MAX_FD_BUFFER_SIZE];
        data_buf[..data.len()].copy_from_slice(data);

        Some(Self {
            header,
            data: data_buf,
            data_len: data.len(),
        })
    }

    pub fn with_remote(mut self, rtr: bool) -> Self {
        self.header.set_rtr(rtr);
        self
    }

    pub fn with_bit_rate_switched(mut self, brs: bool) -> Self {
        self.header.set_brs(brs);
        self
    }

    pub fn with_error_status_indicator(mut self, esi: bool) -> Self {
        self.header.set_esi(esi);
        self
    }

    pub fn with_sequence_number(mut self, seq: u32) -> Self {
        self.header.set_seq(seq);
        self
    }

    pub fn header(&self) -> &TxHeader<[u32; HEADER_SIZE_DWORDS]> {
        &self.header
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..self.data_len]
    }

    #[allow(clippy::identity_op)]
    pub fn as_bytes(&self) -> (usize, [u8; HEADER_SIZE_DWORDS * 4 + MAX_FD_BUFFER_SIZE]) {
        let mut buffer = [0u8; HEADER_SIZE_DWORDS * 4 + MAX_FD_BUFFER_SIZE];

        buffer[0..4].copy_from_slice(&self.header.0[0].to_le_bytes());
        buffer[4..8].copy_from_slice(&self.header.0[1].to_le_bytes());

        buffer[8..self.data.len() + 8].copy_from_slice(&self.data);

        (
            len_for_dlc(self.header.dlc(), self.header.fdf()).unwrap() + 8,
            buffer,
        )
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TxEventObject {
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub header: TxHeader<[u32; HEADER_SIZE_DWORDS]>,
    pub timestamp: Option<u32>,
}
