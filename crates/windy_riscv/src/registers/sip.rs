//! The supervisor interrupt pending register.

use register::{cpu::RegisterReadWrite, register_bitfields};

read_csr!(0x144);
write_csr!(0x144);

// the `u64` here is safe, because we can only build on 64-bit systems
register_bitfields! { u64,
    /// The `sip` register contains information about pending interrupts.
    ///
    /// **Note** that only `SSIP`, `USIP`, and `UEIP` are writable,
    /// all other bits are read-only.
    pub SIP [
        /// Indicates if a User Software interrupt is pending.
        ///
        /// **This bit is readable and writable.**
        USIP OFFSET(0)  NUMBITS(1) [],
        /// Indicates if a Supervisor Software interrupt is pending.
        ///
        /// **This bit is readable and writable.**
        SSIP OFFSET(1)  NUMBITS(1) [],

        /// Indicates if a User Timer interrupt is pending.
        ///
        /// **This bit is read only.**
        UTIP OFFSET(4)  NUMBITS(1) [],
        /// Indicates if a Supervisor Timer interrupt is pending.
        ///
        /// **This bit is read only.**
        STIP OFFSET(5)  NUMBITS(1) [],

        /// Indicates if a User External interrupt is pending.
        ///
        /// **This bit is readable and writable.**
        UEIP OFFSET(8)  NUMBITS(1) [],
        /// Indicates if a Supervisor External interrupt is pending.
        ///
        /// **This bit is read only.**
        SEIP OFFSET(9)  NUMBITS(1) []
    ]
}

pub struct SipRegister;

impl RegisterReadWrite<u64, SIP::Register> for SipRegister {
    #[inline]
    fn get(&self) -> u64 {
        unsafe { _read() as u64 }
    }

    #[inline]
    fn set(&self, value: u64) {
        unsafe { _write(value as usize) };
    }
}

static REGISTER: SipRegister = SipRegister;

/// Returns a reference to a struct that can be used to
/// modify, read, write the register using the [`register`] crate.
///
/// [`register`]: https://docs.rs/register
pub fn sip() -> &'static SipRegister {
    &REGISTER
}
