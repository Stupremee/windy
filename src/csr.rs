//! RISC-V CSR registers.

#[macro_use]
mod macros;

pub mod mie;

pub mod mscratch {
    use crate::trap::TrapFrame;

    write_csr!(0x340);

    /// Write the address for the global trap frame into `mscratch`.
    #[inline]
    pub fn write(ptr: *mut TrapFrame) {
        unsafe {
            _write(ptr as _);
        }
    }
}

pub mod mtvec {
    read_csr!(usize, 0x305);
    write_csr!(pub 0x305);
}
