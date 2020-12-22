use crate::trap::TrapFrame;

write_csr!(0x305);

/// Write the address for the global trap frame into `mscratch`.
#[inline]
pub fn write(ptr: *mut TrapFrame) {
    unsafe {
        _write(ptr as _);
    }
}
