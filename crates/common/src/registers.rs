//! All important registers and abstractions to access them.

#[macro_use]
mod macros;

pub mod mie;

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
