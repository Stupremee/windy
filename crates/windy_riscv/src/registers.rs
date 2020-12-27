//! All important registers and abstractions to access them.

#[macro_use]
mod macros;

pub mod sip;
pub use sip::sip;

pub mod mie;
pub use mie::mie;

/// The `mtvec` CSR.
pub mod mtvec {
    read_csr!(
        /// Reads the raw value from the `mtvec` register.
        pub 0x305
    );

    write_csr!(
        /// Writes the raw value into the `mtvec` register.
        pub 0x305
    );
}

pub mod mvendorid {
    read_csr!(
        /// Reads the raw value from the `mvendorid` register.
        pub 0xF11
    );
}

pub mod marchid {
    read_csr!(
        /// Reads the raw value from the `marchid` register.
        pub 0xF12
    );
}

pub mod mimpid {
    read_csr!(
        /// Reads the raw value from the `mimpid` register.
        pub 0xF13
    );
}
