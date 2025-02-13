pub mod rx;
pub mod tx;

/// The length in DWORDs of the TX and RX header objects
const HEADER_SIZE_DWORDS: usize = 2;

/// The maximum data buffer (paylod) size in bytes
pub const MAX_FD_BUFFER_SIZE: usize = 64;

pub fn dlc_for_len(len: usize, is_fd: bool) -> Option<u8> {
    if is_fd {
        Some(match len {
            0..=8 => len as u8,
            12 => 9,
            16 => 10,
            20 => 11,
            24 => 12,
            32 => 13,
            48 => 14,
            64 => 15,
            _ => return None,
        })
    } else {
        if len > 8 {
            return None;
        }

        Some(len as u8)
    }
}

pub fn len_for_dlc(dlc: u8, is_fd: bool) -> Option<usize> {
    if is_fd {
        Some(match dlc {
            0..=8 => dlc as usize,
            9 => 12,
            10 => 16,
            11 => 20,
            12 => 24,
            13 => 32,
            14 => 48,
            15 => 64,
            _ => return None,
        })
    } else {
        match dlc {
            0..=8 => Some(dlc as usize),
            9..=15 => Some(8),
            _ => None,
        }
    }
}
