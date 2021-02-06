#![allow(unused)]

macro_rules! write_csr {
    ($(#[$meta:meta])* pub $number:expr) => {
        write_csr!($number);

        /// Writes the raw valuue into this CSR.
        $(#[$meta])*
        pub fn write(bits: usize) {
            unsafe { _write(bits) };
        }
    };

    ($number:expr) => {
        /// Writes the raw value into this CSR.
        #[inline(always)]
        unsafe fn _write(bits: usize) {
            asm!("csrw {}, {}", const $number, in(reg) bits);
        }
    };
}

macro_rules! read_csr {
    ($(#[$meta:meta])* pub $number:expr) => {
        read_csr!($number);

        /// Read the raw bits out of this CSR.
        $(#[$meta])*
        pub fn read() -> usize {
            unsafe { _read() }
        }
    };

    ($number:expr) => {
        /// Read the raw bits out of a CSR.
        #[inline(always)]
        unsafe fn _read() -> usize {
            let bits;
            asm!("csrr {}, {}", out(reg) bits, const $number);
            bits
        }
    };
}

macro_rules! set_csr {
    ($(#[$meta:meta])* pub $number:expr) => {
        set_csr!($number);

        /// Set all bits specified by the mask to one inside this CSR.
        $(#[$meta])*
        pub fn set(mask: usize) {
            unsafe { _set(mask) };
        }
    };

    ($number:expr) => {
        #[inline(always)]
        unsafe fn _set(mask: usize) {
            asm!("csrs {}, {}", const $number, in(reg) mask);
        }
    };
}

macro_rules! clear_csr {
    ($(#[$meta:meta])* pub $number:expr) => {
        clear_csr!($number);

        /// Clear all bits specified by the mask inside this CSR.
        $(#[$meta])*
        pub fn clear(mask: usize) {
            unsafe { _clear(mask) }
        }
    };

    ($number:expr) => {
        #[inline(always)]
        unsafe fn _clear(mask: usize) {
            asm!("csrc {}, {}", const $number, in(reg) mask);
        }
    };
}

macro_rules! csr_mod {
    (rw, $name:ident, $num:expr) => {
        #[doc = concat!("The `", stringify!($name), "` CSR.")]
        pub mod $name {
            read_csr!(
                #[doc = concat!("Reads the raw value from the `", stringify!($name), "` register.")]
                pub $num
            );

            write_csr!(
                #[doc = concat!("Writes the raw value into the `", stringify!($name), "` register.")]
                pub $num
            );
        }
    };

    (r, $name:ident, $num:expr) => {
        #[doc = concat!("The `", stringify!($name), "` CSR.")]
        pub mod $name {
            read_csr!(
                #[doc = concat!("Reads the raw value from the `", stringify!($name), "` register.")]
                pub $num
            );
        }
    };
}
