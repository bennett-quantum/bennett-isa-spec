//! Snapshot allocation work must not grow with immutable per-wire metadata.
use bennett_isa_ref::QuantumRegisters;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    static ALLOCATIONS: Cell<Option<usize>> = const { Cell::new(None) };
}

struct CountingAllocator;

fn count_allocation() {
    ALLOCATIONS.with(|count| {
        if let Some(value) = count.get() {
            count.set(Some(value + 1));
        }
    });
}

// Each request is forwarded unchanged to the system allocator. Thread-local
// counting excludes the test harness and other concurrently executing tests.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count_allocation();
        System.alloc(layout)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        count_allocation();
        System.alloc_zeroed(layout)
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        count_allocation();
        System.realloc(pointer, layout, size)
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        System.dealloc(pointer, layout);
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn snapshot_allocations(width: usize) -> usize {
    let mut registers = QuantumRegisters::new();
    let id = registers.allocate_in_class(width, "any").unwrap();
    ALLOCATIONS.with(|count| count.set(Some(0)));
    let mut snapshot = std::hint::black_box(registers.clone());
    let allocations = ALLOCATIONS.with(|count| count.replace(None).unwrap());

    assert_eq!(snapshot, registers);
    assert_eq!(snapshot.classes(id).unwrap().len(), width);
    assert!(snapshot.classes(id).unwrap().iter().all(|c| c == "any"));
    snapshot.free(id).unwrap();
    assert!(snapshot.bundle(id).is_err());
    assert_eq!(registers.bundle(id).unwrap().len(), width);
    allocations
}

#[test]
fn snapshot_allocation_count_is_independent_of_register_width() {
    const NARROW_WIDTH: usize = 1;
    const WIDE_WIDTH: usize = 64;
    let narrow = snapshot_allocations(NARROW_WIDTH);
    let wide = snapshot_allocations(WIDE_WIDTH);
    eprintln!("register snapshot allocations: narrow={narrow}, wide={wide}");
    assert_eq!(
        wide, narrow,
        "snapshot cloning must not allocate separately for each wire's immutable class"
    );
}
