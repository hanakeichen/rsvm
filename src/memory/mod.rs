use std::cmp::{PartialEq, PartialOrd};

pub mod heap;
pub mod space;

pub const KB: usize = 1024;
pub const MB: usize = 1024 * KB;
pub const GB: usize = 1024 * MB;

// pub type Address = *const u8;

#[derive(PartialOrd, PartialEq, Clone, Copy)]
pub struct Address(*const u8);

impl Address {
    pub const fn new(ptr: *const u8) -> Self {
        Address(ptr)
    }

    pub const fn from_usize(addr: usize) -> Self {
        Address(addr as _)
    }

    pub const fn null() -> Self {
        Self::new(std::ptr::null())
    }

    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }

    pub fn uoffset(&self, size: usize) -> Address {
        self.offset(size as isize)
    }

    pub fn offset(&self, size: isize) -> Address {
        unsafe { Address::new(self.0.offset(size)) }
    }

    pub const fn ptr(&self) -> *const u8 {
        self.0
    }
}

fn align_of(size: usize, align: usize) -> usize {
    assert!(is_power_of_2(align as isize));
    return ((size + ((1 << align) - 1)) >> align) << align;
}

fn is_align_of(size: usize, align: usize) -> bool {
    (size & (align - 1)) == 0
}

fn is_power_of_2(val: isize) -> bool {
    (val & (val - 1)) == 0
}
