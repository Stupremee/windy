//! RFENCE SBI extension implementation

use crate::{Error, SbiResult};

pub(super) fn handle_ecall(fid: u32) -> SbiResult<usize> {
    match fid {
        _ => Err(Error::NotSupported),
    }
}
