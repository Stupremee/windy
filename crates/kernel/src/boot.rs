use crate::{
    console,
    page::{sv39::Table, PageSize, Perm},
    pmem, StaticCell,
};
use devicetree::DeviceTree;
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
    if console::init(&tree) {
        info!("{} Uart console", "Initialized".green());
    }

    // make the physical memory allocator ready for allocation
    let heap = pmem::init(&tree).expect("failed to initialize the physical memory allocator");

    // set up mapping
    let table = &mut *ROOT_TABLE.get();

    // identity map the regions of the page allocator
    for range in heap.as_slice() {
        let start = range.start as usize;
        let end = range.end as usize;

        table
            .identity_map(
                start.into(),
                end.into(),
                Perm::READ | Perm::WRITE,
                PageSize::Kilopage,
            )
            .expect("failed to map heap region");
    }

    // identity map all sections
    let mut map_section = |(start, end): (*mut u8, *mut u8), perm: Perm| {
        table
            .identity_map(start.into(), end.into(), perm, PageSize::Kilopage)
            .expect("failed to map kernel section");
    };

    map_section(
        dbg!(symbols::kernel_range()),
        Perm::READ | Perm::WRITE | Perm::EXEC,
    );
    //map_section(symbols::text_range(), Perm::READ | Perm::EXEC);
    //map_section(symbols::rodata_range(), Perm::READ);
    //map_section(symbols::data_range(), Perm::READ | Perm::WRITE);
    //map_section(symbols::bss_range(), Perm::READ | Perm::WRITE);
    //map_section(symbols::stack_range(), Perm::READ | Perm::WRITE);

    println!("{:x?}", table.translate(0x0000000080200000.into()));

    // enable paging
    let satp = satp::Satp {
        mode: satp::Mode::Sv39,
        asid: 0,
        root_table: &ROOT_TABLE as *const _ as u64,
    };

    satp::write(satp);
    riscv::asm::sfence(None, None);
    //dbg!();

    // jump to the kernel main function
    //crate::kinit(hart, &tree)
    loop {}
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
