use std::alloc::{alloc, Layout};

const INITIAL_CAPACITY: usize = 256;
const DEFAULT_ALIGNMENT: usize = 8;

#[repr(C)]
pub struct Stack {
    base_pointer: *mut u8,
    size: usize,
    capacity: usize,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            base_pointer: unsafe {
                alloc(Layout::from_size_align(INITIAL_CAPACITY, DEFAULT_ALIGNMENT).unwrap())
            },
            size: 0,
            capacity: INITIAL_CAPACITY,
        }
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}