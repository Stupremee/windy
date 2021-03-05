use crate::{
    console,
    page::{sv39::Table, PageSize, Perm},
    pmem, StaticCell,
};
use devicetree::DeviceTree;
use pmem::alloc::PAGE_SIZE;
use riscv::{csr::satp, symbols};

static ROOT_TABLE: StaticCell<Table> = StaticCell::new(Table::new());

/// Function that is run before `kinit` which is meant to setup paging and stuff
/// and then jumps into `kinit`.
#[no_mangle]
unsafe extern "C" fn _before_main(hart: usize, fdt: *const u8) -> ! {
    // parse the device tree that is later used to initialize certain devices
    let tree = DeviceTree::from_ptr(fdt);
    let tree = tree.expect("failed to initialize devicetree");

    // try to initialize uart debugging
    let uart_addr = console::init(&tree).map(|x| {
        info!("{} Uart console", "Initialized".green());
        x
    });

    // make the physical memory allocator ready for allocation
    let heap = pmem::init(&tree).expect("failed to initialize the physical memory allocator");

    // set up mapping
    let table = &mut *ROOT_TABLE.get();

    // map the device tree
    let len = pmem::alloc::align_up(tree.total_size() as usize, PAGE_SIZE);
    table
        .identity_map(
            fdt.into(),
            (fdt as usize + len).into(),
            Perm::READ,
            PageSize::Kilopage,
        )
        .expect("failed to map device tree");

    // identity map the regions of the page allocator
    for range in heap.as_slice() {
        let start = range.start as usize;
        let end = range.end as usize + 1;

        table
            .fit_identity_map(start.into(), end.into(), Perm::READ | Perm::WRITE)
            .expect("failed to map heap region");
    }

    // identity map all sections
    let mut map_section = |(start, end): (*mut u8, *mut u8), perm: Perm| {
        table
            .fit_identity_map(start.into(), end.into(), perm)
            .expect("failed to map kernel section");
    };

    map_section(symbols::text_range(), Perm::READ | Perm::EXEC);
    map_section(symbols::rodata_range(), Perm::READ);
    map_section(symbols::data_range(), Perm::READ | Perm::WRITE);
    map_section(symbols::bss_range(), Perm::READ | Perm::WRITE);
    map_section(symbols::stack_range(), Perm::READ | Perm::WRITE);

    // map uart mmio device
    if let Some(uart) = uart_addr {
        table
            .map(
                uart.into(),
                uart.into(),
                PageSize::Kilopage,
                Perm::READ | Perm::WRITE,
            )
            .expect("failed to map uart driver");
    }

    table
        .map(
            0x10_0000.into(),
            0x10_0000.into(),
            PageSize::Kilopage,
            Perm::WRITE | Perm::READ,
        )
        .unwrap();

    // enable paging
    let satp = satp::Satp {
        mode: satp::Mode::Sv39,
        asid: 0,
        root_table: &ROOT_TABLE as *const _ as u64,
    };

    satp::write(satp);
    riscv::asm::sfence(None, None);

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
