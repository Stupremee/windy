//! The `mtvec` CSR.

use register::{cpu::RegisterReadWrite, register_bitfields};

read_csr!(0x304);
write_csr!(0x304);

// the `u64` here is safe, because we can only build on 64-bit systems
register_bitfields! { u64,
    pub MIE [
        /// Enables U-Mode software interrupts.
        USIE OFFSET(0)  NUMBITS(1) [],
        /// Enables S-Mode software interrupts.
        SSIE OFFSET(1)  NUMBITS(1) [],
        /// Enables M-Mode software interrupts.
        MSIE OFFSET(3)  NUMBITS(1) [],

        /// Enables U-Mode timer interrupts.
        UTIE OFFSET(4)  NUMBITS(1) [],
        /// Enables S-Mode timer interrupts.
        STIE OFFSET(5)  NUMBITS(1) [],
        /// Enables M-Mode timer interrupts.
        MTIE OFFSET(7)  NUMBITS(1) [],

        /// Enables U-Mode external interrupts.
        UEIE OFFSET(8)  NUMBITS(1) [],
        /// Enables S-Mode external interrupts.
        SEIE OFFSET(9)  NUMBITS(1) [],
        /// Enables M-Mode external interrupts.
        MEIE OFFSET(11)  NUMBITS(1) []
    ]
}

pub struct MieRegister;

impl RegisterReadWrite<u64, MIE::Register> for MieRegister {
    #[inline]
    fn get(&self) -> u64 {
        unsafe { _read() as u64 }
    }

    #[inline]
    fn set(&self, value: u64) {
        unsafe { _write(value as usize) };
    }
}

static REGISTER: MieRegister = MieRegister;

/// Returns a reference to a struct that can be used to
/// modify, read, write the register using the [`register`] crate.
///
/// [`register`]: https://docs.rs/register
pub fn mie() -> &'static MieRegister {
    &REGISTER
}
