//! Utilities for working with raw byte units.

use core::fmt;

/// `1 KiB`
pub const KIB: usize = 1 << 10;
/// `1 MiB`
pub const MIB: usize = 1 << 20;
/// `1 GiB`
pub const GIB: usize = 1 << 30;
/// `1 TiB`
pub const TIB: usize = 1 << 40;

/// Wrapper around raw byte that pretty-prints
/// them using the [`Display`](core::fmt::Display)
/// implementation.
#[derive(Debug, Clone, Copy)]
pub struct ByteUnit(pub usize);

impl fmt::Display for ByteUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let count = self.0 as f32;

        match self.0 {
            0..KIB => write!(f, "{} B", self.0)?,
            0..MIB => write!(f, "{:.2} KiB", count / KIB as f32)?,
            0..GIB => write!(f, "{:.2} MiB", count / MIB as f32)?,
            0..TIB => write!(f, "{:.2} GiB", count / GIB as f32)?,
            _ => write!(f, "{:.2} TiB", count / TIB as f32)?,
        };

        Ok(())
    }
}
