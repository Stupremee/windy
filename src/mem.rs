//! Implementation of the Memory System for the kernel

pub mod linked_list;

mod buddy;
mod slab;

pub use buddy::BuddyAllocator;
pub use linked_list::LinkedList;
pub use slab::Slab;

use core::{
    alloc::{AllocError, Layout},
    fmt,
    lazy::Lazy,
    ptr::{self, NonNull},
};
use spin::Mutex;

/// The size of a single memory page is 4KiB,
/// this is also the size order-0 in the buddy
/// allocator.
pub const PAGE_SIZE: usize = 1 << 12;

/// Statistics of any kind of allocator.
#[derive(Debug, Clone)]
pub struct AllocStats {
    /// The name of the allocator these stats belong to.
    pub name: &'static str,
    /// The bytes that were actuallly requested by the user.
    pub requested: usize,
    /// The bytes that were actually allocated.
    pub allocated: usize,
    /// The total number of bytes this allocator can reach.
    pub total: usize,
}

impl AllocStats {
    /// Create a new [`AllocStats`] instance for the given allocator name.
    pub fn with_name(name: &'static str) -> Self {
        Self {
            name,
            requested: 0,
            allocated: 0,
            total: 0,
        }
    }
}

impl fmt::Display for AllocStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.name)?;
        self.name.chars().try_for_each(|_| write!(f, "~"))?;
        writeln!(f, "\nRequested bytes: 0x{:x}", self.requested)?;
        writeln!(f, "Allocated bytes: 0x{:x}", self.allocated)?;
        writeln!(f, "Total bytes:     0x{:x}", self.total)?;
        self.name.chars().try_for_each(|_| write!(f, "~"))?;
        writeln!(f)?;
        Ok(())
    }
}

/// Returns the range of the heap defined by the linker script
///
/// Returns `(start, end)` pointers
pub fn heap_range() -> (NonNull<u8>, NonNull<u8>) {
    let start = crate::utils::heap_start();
    // SAFETY
    // The heap values come from the linker, so they must
    // be valid.
    let end = unsafe { start.add(crate::utils::heap_size()) };

    let start = NonNull::new(start as *mut _).expect("Heap start pointer was NULL");
    let end = NonNull::new(end as *mut _).expect("Heap end pointer was NULL");

    (start, end)
}

/// The central allocator for the kernel.
///
/// It's combination of a buddy and slab allocator.
/// It will contain 7 slabs: 64B, 128B, 256B, 1KiB, 2KiB, 4KiB; and a [`BuddyAllocator`].
/// The memory for the slabs will be allocated by the [`BuddyAllocator`].
/// Allocations that are too large to be allocated by the slabs, will be redirected to the
/// internal [`BuddyAllocator`].
pub struct Allocator {
    slab_64: Slab,
    slab_128: Slab,
    slab_256: Slab,
    slab_512: Slab,
    slab_1024: Slab,
    slab_2048: Slab,
    slab_4096: Slab,
    buddy: BuddyAllocator,

    stats: AllocStats,
}

impl Allocator {
    /// Creates an uninitialized [`Allocator`].
    pub const fn new() -> Self {
        Self {
            slab_64: Slab::new(),
            slab_128: Slab::new(),
            slab_256: Slab::new(),
            slab_512: Slab::new(),
            slab_1024: Slab::new(),
            slab_2048: Slab::new(),
            slab_4096: Slab::new(),
            buddy: BuddyAllocator::new(),
            stats: AllocStats::with_name("Allocator"),
        }
    }

    /// Initializes this [`Allocator`] with the given heap range.
    pub unsafe fn init(&mut self, start: NonNull<u8>, end: NonNull<u8>) {
        assert!(start < end, "start and end are in wrong order");

        // buddy allocator will be initialized first, because we need
        // it for everything else.
        self.buddy.add_heap(start, end);

        // get the total heap size, and calculate the size for each allocator.
        let heap_size = end.as_ptr() as usize - start.as_ptr() as usize;
        // we use `next_power_of_two` here, because we want aligned and even memory
        // for our slabs, and the rest of the heap that will not be covered by
        // the slabs, will be covered by the buddy allocator.
        let slab_size = heap_size.next_power_of_two() / 8;

        // closure for alllocating space for a slab, and then initializing the slab.
        let mut init_slab = |block_size: usize| {
            let ptr = self
                .buddy
                .alloc_size(slab_size)
                .expect("failed to allocate memory for slab");
            slab.init(ptr, block_size, slab_size);
        };

        init_slab(&mut self.slab_64, 64);
        init_slab(&mut self.slab_128, 128);
        init_slab(&mut self.slab_256, 256);
        init_slab(&mut self.slab_512, 512);
        init_slab(&mut self.slab_1024, 1024);
        init_slab(&mut self.slab_2048, 2048);
        init_slab(&mut self.slab_4096, 4096);
    }

    /// Allocates enough memory for the given `layout`.
    ///
    /// Returns a pointer to the chunk of memory where `layout` can be put in.
    pub fn allocate(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
        match Self::alloc_for_layout(layout) {
            AllocatorKind::Slab64 => self.slab_64.alloc(),
            AllocatorKind::Slab128 => self.slab_128.alloc(),
            AllocatorKind::Slab256 => self.slab_256.alloc(),
            AllocatorKind::Slab512 => self.slab_512.alloc(),
            AllocatorKind::Slab1024 => self.slab_1024.alloc(),
            AllocatorKind::Slab2048 => self.slab_2048.alloc(),
            AllocatorKind::Slab4096 => self.slab_4096.alloc(),
            AllocatorKind::Buddy => self.buddy.alloc_size(layout.size()),
        }
    }

    /// Frees a block of memory that was previously allocated by this
    /// allocator.
    ///
    /// # Safety
    /// - `ptr` must be allocated by the same allocator instance.
    /// - `layout` must be the layout that the block was allocated with
    pub unsafe fn deallocate(&mut self, ptr: NonNull<u8>, layout: Layout) {
        match Self::alloc_for_layout(layout) {
            AllocatorKind::Slab64 => self.slab_64.dealloc(ptr),
            AllocatorKind::Slab128 => self.slab_128.dealloc(ptr),
            AllocatorKind::Slab256 => self.slab_256.dealloc(ptr),
            AllocatorKind::Slab512 => self.slab_512.dealloc(ptr),
            AllocatorKind::Slab1024 => self.slab_1024.dealloc(ptr),
            AllocatorKind::Slab2048 => self.slab_2048.dealloc(ptr),
            AllocatorKind::Slab4096 => self.slab_4096.dealloc(ptr),
            AllocatorKind::Buddy => self.buddy.dealloc_size(ptr, layout.size()),
        }
    }

    fn alloc_for_layout(layout: Layout) -> AllocatorKind {
        use AllocatorKind::*;

        match (layout.size(), layout.align()) {
            (..=64, ..=64) => Slab64,
            (..=128, ..=128) => Slab128,
            (..=256, ..=256) => Slab256,
            (..=512, ..=512) => Slab512,
            (..=1024, ..=1024) => Slab1024,
            (..=2048, ..=2048) => Slab2048,
            (..=4096, ..=4096) => Slab4096,
            (_, _) => Buddy,
        }
    }
}

/// Internal enum to differentiate between all different Allocators.
enum AllocatorKind {
    Slab64,
    Slab128,
    Slab256,
    Slab512,
    Slab1024,
    Slab2048,
    Slab4096,
    Buddy,
}

#[global_allocator]
static ALLOCATOR: GlobalAllocator = GlobalAllocator::new();

/// Wrapper around an [`Allocator`] that can be used in a global context,
/// because it's synchronized using a `Mutex`.
pub struct GlobalAllocator(Mutex<Allocator>);

unsafe impl Send for GlobalAllocator {}
unsafe impl Sync for GlobalAllocator {}

impl GlobalAllocator {
    /// Creates a new `GlobalAllocator`.
    const fn new() -> Self {
        Self(Mutex::new(Allocator::new()))
    }
}

unsafe impl core::alloc::Allocator for GlobalAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        assert_ne!(
            layout.size(),
            0,
            "ZST types are not supported by the allocator"
        );
        let ptr = self.0.lock().allocate(layout)?;
        Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.0.lock().deallocate(ptr, layout)
    }
}

unsafe impl core::alloc::GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0
            .lock()
            .allocate(layout)
            .map(NonNull::as_ptr)
            .unwrap_or(ptr::null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        todo!()
    }
}
