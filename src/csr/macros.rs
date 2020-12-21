macro_rules! read_csr {
    (impl $name:ident, $number:expr) => {
        read_csr!($name);

        #[inline]
        pub fn read() -> $name {
            let bits = _read();
            Self { bits }
        }
    };

    ($name:ident, $number:expr) => {
        read_csr!($number);

        #[derive(Debug, Clone, Copy)]
        pub struct $name {
            bits: usize
        }

        #[inline]
        pub fn read() -> $name {
            let bits = unsafe { _read() };
            $name { bits }
        }
    };

    ($number:expr) => {
        /// Read the raw value out of the CSR.
        #[inline]
        unsafe fn _read() -> usize {
            let bits: usize;
            asm!("csrrs {}, {}, x0", out(reg) bits, const $number, options(nostack));
            bits
        }
    };
}

macro_rules! write_csr {
    (pub $number:expr) => {
        write_csr!($number);

        /// Write the raw bits into the register.
        #[inline]
        pub fn write(bits: usize) {
            unsafe { _write() }
        }
    };

    ($number:expr) => {
        /// Write the raw bits into the register.
        #[inline]
        unsafe fn _write(bits: usize) {
            asm!("csrrw x0, {}, {}", const $number, in(reg) bits, options(nostack));
        }
    };
}
