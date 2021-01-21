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

macro_rules! csr_bits {
    ($num:expr, $(
        $(#[$attr:meta])*
        $perm:ident $name:ident: $from:literal $(.. $to:literal =
            $(#[$kind_attr:meta])*
            $kind_name:ident [
                $(
                    $(#[$kind_variant_attr:meta])*
                    $kind_variant:ident = $kind_val:expr
                ),*$(,)?
            ]
        )?
    ),* $(,)?) => {
        $($(
            $(#[$kind_attr])*
            #[derive(Clone, Copy, Debug)]
            pub enum $kind_name {
                $(
                    $(#[$kind_variant_attr])*
                    $kind_variant
                ),*
            }
        )*)?

        $(
            ::paste::paste! {
                $(#[$attr])*
                pub mod [< $name:lower >] {
                    csr_bits!(@single_bit, $num, $perm $name: $from);
                }
            }
        )*
    };

    ($num:expr, $(
        $(#[$attr:meta])*
        $perm:ident $name:ident: $from:literal to $to:literal =
            $(#[$kind_attr:meta])*
            $kind_name:ident [
                $(
                    $(#[$kind_variant_attr:meta])*
                    $kind_variant:ident = $kind_val:expr
                ),*$(,)?
            ]
    ),* $(,)?) => {
        $(
            $(#[$kind_attr])*
            #[derive(Clone, Copy, Debug)]
            pub enum $kind_name {
                $(
                    $(#[$kind_variant_attr])*
                    $kind_variant
                ),*
            }
        )*

        //$(
            //$(#[$attr])*
            //#[allow(non_snake_case)]
            //pub mod $name {
                //csr_bits!(@single_bit, $num, $perm $name: $bit);
            //}
        //)*
    };

    (@single_bit, $num:expr, rw $name:ident: $bit:literal) => {
        /// Set the single bit to `true`/`1`.
        pub fn set() {
            let bit = 1 << $bit;
            unsafe {
                asm!("csrs {csr} {}", inout(reg) bit => _, csr = const $num);
            }
        }

        csr_bits!(@single_bit, $num, r $name: $bit);
    };

    (@single_bit, $num:expr, r $name:ident: $bit:literal) => {
        /// Get the value of this bit.
        pub fn get() -> bool {
            let bit: usize;
            unsafe {
                asm!("csrr {} {csr}", out(reg) bit, csr = const $num);
            }

            bit & (1 << $bit) != 0
        }
    };
}
