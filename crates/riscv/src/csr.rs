//! Safe access to all CSRs and

#[macro_use]
mod macros;

csr_mod!(r, mvendorid, 0xF11);
csr_mod!(r, marchid, 0xF12);
csr_mod!(r, mimpid, 0xF13);
csr_mod!(r, mhartid, 0xF14);

csr_mod!(r, satp, 0x180);
