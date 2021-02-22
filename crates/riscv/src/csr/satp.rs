//! The `satp` CSR.

write_csr!(0x180);
read_csr!(0x180);

/// The paging mode to set inside the satp register.
#[derive(Clone, Debug)]
pub enum Mode {
    Bare,
    Sv39,
    Sv48,
}

/// An abstraction around the bitfield of the `satp` register.
#[derive(Clone, Debug)]
pub struct Satp {
    pub mode: Mode,
    pub asid: u16,
    pub root_table: u64,
}

/// Read from the `satp` CSR.
pub fn read() -> Satp {
    let bits = unsafe { _read() };

    let mode = match bits >> 60 {
        0 => Mode::Bare,
        8 => Mode::Sv39,
        9 => Mode::Sv48,
        _ => panic!("unimplemented page table mode"),
    };

    Satp {
        mode,
        asid: ((bits >> 44) & 0xFFFF) as u16,
        root_table: ((bits & 0xFFF_FFFF_FFFF) << 12) as u64,
    }
}

/// Write to the `satp` CSR.
pub fn write(satp: Satp) {
    let bits = (satp.root_table >> 12) | ((satp.asid as u64) << 44);

    let mode = match satp.mode {
        Mode::Bare => 0,
        Mode::Sv39 => 8,
        Mode::Sv48 => 9,
    };

    let bits = bits | (mode << 60);
    unsafe { _write(bits as usize) }
}
