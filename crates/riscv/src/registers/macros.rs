macro_rules! write_csr {
    ($(#[$meta:meta])* pub $number:expr) => {
        write_csr!($number);

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

macro_rules! csr_mod {
    (rw, $name:ident, $num:expr) => {
        ::paste::paste! {
            #[doc = "The `" $name "` CSR."]
            pub mod $name {
                read_csr!(
                    #[doc = "Reads the raw value from the `" $name "` register."]
                    pub $num
                );

                write_csr!(
                    #[doc = "Writes the raw value into the `" $name "` register."]
                    pub $num
                );
            }
        }
    };

    (r, $name:ident, $num:expr) => {
        ::paste::paste! {
            #[doc = "The `" $name "` CSR."]
            pub mod $name {
                read_csr!(
                    #[doc = "Reads the raw value from the `" $name "` register."]
                    pub $num
                );
            }
        }
    };
}
