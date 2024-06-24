use bitfield::bitfield;

use crate::memory::controller::filter::FilterNumber;

use super::{len_for_dlc, HEADER_SIZE_DWORDS, MAX_FD_BUFFER_SIZE};

bitfield! {
    pub struct RxHeader([u32]);
    impl Debug;
    u8;

    /* T0 */

    /// Standard ID
    pub u16, sid, _: 10, 0;
    /// Extended ID
    pub u32, eid, _: 28, 11;
    sid11, _: 29;

    /* T1 */

    /// Data Length Code
    pub dlc, _: 35, 32;
    /// ID Extension
    pub ide, _: 36;
    /// Remote Transmission Request
    pub rtr, _: 37;
    /// Bit Rate Switched
    pub brs, _: 38;
    /// FD Frame
    pub fdf, _: 39;
    /// Error Status Indicator
    pub esi, _: 40;
    /// Filter Hit (number of the filter that matched)
    _filhit, _: 47, 43;
}

impl<T: AsRef<[u32]>> RxHeader<T> {
    /// Returns which filter was matched when receiving this message
    pub fn filter_hit(&self) -> FilterNumber {
        self._filhit().try_into().unwrap()
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RxMessage {
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    header: RxHeader<[u32; HEADER_SIZE_DWORDS]>,
    timestamp: Option<u32>,
    data: [u8; MAX_FD_BUFFER_SIZE],
}

impl RxMessage {
    /// Constructs a new RxMessage from the data found in the chip's RAM
    pub fn new(
        header: RxHeader<[u32; HEADER_SIZE_DWORDS]>,
        timestamp: Option<u32>,
        data: &[u8],
    ) -> Option<RxMessage> {
        if data.len() > MAX_FD_BUFFER_SIZE {
            return None;
        }

        let mut buffer = [0u8; MAX_FD_BUFFER_SIZE];
        buffer[..data.len()].copy_from_slice(data);

        Some(Self {
            header,
            timestamp,
            data: buffer,
        })
    }

    /// Gets the message header to inspect the low level control bits
    pub fn header(&self) -> &RxHeader<[u32; HEADER_SIZE_DWORDS]> {
        &self.header
    }

    /// Gets the message timestamp if the FIFO was configured to include one
    pub fn timestamp(&self) -> Option<u32> {
        self.timestamp
    }

    /// Creates a slice over the data associated with this message with the
    /// correct length calculated from the DLC
    pub fn data(&self) -> &[u8] {
        &self.data[..len_for_dlc(self.header.dlc(), self.header.fdf()).unwrap()]
    }

    /// Determines from the header whether or not this message is a CAN FD frame
    pub fn is_fd(&self) -> bool {
        self.header.fdf()
    }
}
