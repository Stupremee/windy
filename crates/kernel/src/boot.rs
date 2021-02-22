use crate::{
    console,
    page::{sv39::Table, PageSize, Perm},
    pmem, StaticCell,
};
use devicetree::DeviceTree;
use riscv::csr::satp;

static ROOT_TABLE: StaticCell<Table> = StaticCell::new(Table::new());

/// Function that is run before `kinit` which is meant to setup paging and stuff
/// and then jumps into `kinit`.
#[no_mangle]
unsafe extern "C" fn _before_main(hart: usize, fdt: *const u8) -> ! {
    // parse the device tree that is later used to initialize certain devices
    let tree = DeviceTree::from_ptr(fdt);
    let tree = tree.expect("failed to initialize devicetree");

    // try to initialize uart debugging
    if console::init(&tree) {
        info!("{} Uart console", "Initialized".green());
    }

    // make the physical memory allocator ready for allocation
    pmem::init(&tree).expect("failed to initialize the physical memory allocator");

    // set up mapping
    let table = &mut *ROOT_TABLE.get();

    // identity map the kernel
    let (kernel_start, kernel_end) = riscv::symbols::kernel_range();
    table
        .identity_map(
            kernel_start.into(),
            kernel_end.into(),
            Perm::READ | Perm::WRITE | Perm::EXEC,
            PageSize::Kilopage,
        )
        .expect("failed to map kernel region");

    // identity map all remaining sections
    let (start, end) = riscv::symbols::text_range();
    table
        .identity_map(
            start.into(),
            end.into(),
            Perm::READ | Perm::EXEC,
            PageSize::Kilopage,
        )
        .expect("failed to map `.text` section");

    let (start, end) = riscv::symbols::rodata_range();
    table
        .identity_map(start.into(), end.into(), Perm::READ, PageSize::Kilopage)
        .expect("failed to map `.rodata` section");

    let (start, end) = riscv::symbols::data_range();
    table
        .identity_map(
            start.into(),
            end.into(),
            Perm::READ | Perm::WRITE,
            PageSize::Kilopage,
        )
        .expect("failed to map `.data` section");

    let (start, end) = riscv::symbols::bss_range();
    table
        .identity_map(
            start.into(),
            end.into(),
            Perm::READ | Perm::WRITE,
            PageSize::Kilopage,
        )
        .expect("failed to map`.bss` section");

    let (start, end) = riscv::symbols::stack_range();
    table
        .identity_map(
            start.into(),
            end.into(),
            Perm::READ | Perm::WRITE,
            PageSize::Kilopage,
        )
        .expect("failed to map stack");

    // enable paging
    let satp = satp::Satp {
        mode: satp::Mode::Sv39,
        asid: 0,
        root_table: &ROOT_TABLE as *const _ as u64,
    };

    dbg!();
    satp::write(satp);
    riscv::asm::sfence(None, None);
    dbg!();

    // jump to the kernel main function
    crate::kinit(hart, &tree)
}

/// The entrypoint for the whole kernel.
///
/// `a0` = hart id
/// `a1` = pointer to device tree
#[naked]
#[no_mangle]
#[link_section = ".text.init"]
pub unsafe extern "C" fn _boot() -> ! {
    asm!(
        // ---------------------------------
        // Load the global pointer into
        // the `gp` register
        // ---------------------------------
        ".option push",
        ".option norelax",
        "    la gp, __global_pointer",
        ".option pop",
        // ---------------------------------
        // Disable interrupts
        // ---------------------------------
        "csrw sie, zero",
        "csrci sstatus, 2",
        // ---------------------------------
        // Set `bss` to zero
        // ---------------------------------
        "    la t0, __bss_start",
        "    la t1, __bss_end",
        "    bgeu t0, t1, zero_bss_done",
        "zero_bss:",
        "    sd zero, (t0)",
        "    addi t0, t0, 8",
        "zero_bss_done:",
        // ---------------------------------
        // Initialize stack
        // ---------------------------------
        "    la sp, __stack_end",
        // ---------------------------------
        // Jump into rust code
        // ---------------------------------
        "j _before_main",
        options(noreturn)
    )
}
