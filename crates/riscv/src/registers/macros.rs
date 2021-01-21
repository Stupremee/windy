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
    ($read:ident, $write:ident, $set:ident, $clear:ident, $(
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
                    csr_bits!(@single_bit, $read, $write, $set, $clear, $perm $name: $from $(
                        .. $to = $kind_name [
                            $($kind_variant = $kind_val),*
                        ]
                    )?);
                }
            }
        )*
    };

    (@single_bit, $read:ident, $write:ident, $set:ident, $clear:ident,
        rw $name:ident: $from:literal .. $to:literal = $kind_name:ident [
        $($kind_variant:ident = $kind_val:expr),*$(,)?
    ]) => {
        /// Set the bits for this field to the given value.
        pub fn set(val: super::$kind_name) {
            use $crate::BitField;

            #[allow(unused_unsafe)]
            let mut bits = unsafe { super::$read() };
            let new_bits = match val {
                $(super::$kind_name::$kind_variant => $kind_val,)*
            };

            bits.set_bits($from..=$to, new_bits);
            #[allow(unused_unsafe)]
            unsafe { super::$write(bits) };
        }

        /// Clear all bits this bitfield covers.
        pub fn clear() {
            use $crate::BitField;

            let mut mask = 0usize;
            let diff = $to - $from;
            mask.set_bits($from..=$to, (1 << diff) - 1);
            #[allow(unused_unsafe)]
            unsafe { super::$clear(mask) };
        }

        csr_bits!(@single_bit, $read, $write, $set, $clear, r $name: $from .. $to = $kind_name [
            $($kind_variant = $kind_val),*
        ]);
    };

    (@single_bit, $read:ident, $write:ident, $set:ident, $clear:ident,
        r $name:ident: $from:literal .. $to:literal = $kind_name:ident [
        $($kind_variant:ident = $kind_val:expr),*$(,)?
    ]) => {
        /// Get the value of this bitfield.
        ///
        /// Returns `None` if the bitfield encoded an invalid value.
        pub fn get() -> Option<super::$kind_name> {
            use $crate::BitField;

            #[allow(unused_unsafe)]
            let bits = unsafe { super::$read() };
            let bits = bits.get_bits($from..=$to);
            match bits {
                $($kind_val => Some(super::$kind_name::$kind_variant),)*
                _ => None,
            }
        }
    };

    (@single_bit, $read:ident, $write:ident, $set:ident, $clear:ident, rw $name:ident: $bit:literal) => {
        /// Set the single bit to `true`/`1`.
        pub fn set() {
            let bit = 1 << $bit;
            #[allow(unused_unsafe)]
            unsafe { super::$set(bit); }
        }

        /// Clear the bit of this bitfield inside the CSR.
        pub fn clear() {
            let bit = 1 << $bit;
            #[allow(unused_unsafe)]
            unsafe { super::$clear(bit); }
        }

        csr_bits!(@single_bit, $read, $write, $set, $clear, r $name: $bit);
    };

    (@single_bit, $read:ident, $_:ident, $__:ident, $___:ident, r $name:ident: $bit:literal) => {
        /// Get the value of this bit.
        pub fn get() -> bool {
            #[allow(unused_unsafe)]
            let bit = unsafe { super::$read() };
            bit & (1 << $bit) != 0
        }
    };
}
