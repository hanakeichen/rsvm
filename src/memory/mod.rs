use std::{cmp::{PartialEq, PartialOrd}, ffi::c_void};

pub mod heap;
pub mod lab;
pub mod space;

pub const KB: usize = 1024;
pub const MB: usize = 1024 * KB;
pub const GB: usize = 1024 * MB;

pub const POINTER_SIZE: usize = std::mem::size_of::<*const u8>();

// pub type Address = *const u8;

#[derive(PartialOrd, PartialEq, Clone, Copy, Debug)]
pub struct Address(*const u8);

impl Address {
    #[inline(always)]
    pub const fn new(ptr: *const u8) -> Self {
        Address(ptr)
    }

    #[inline(always)]
    pub const fn from_ref<T>(v: &T) -> Self {
        Address(v as *const T as _)
    }

    #[inline(always)]
    pub const fn from_usize(addr: usize) -> Self {
        Address(addr as _)
    }

    #[inline(always)]
    pub const fn from_isize(addr: isize) -> Self {
        Address(addr as _)
    }

    #[inline(always)]
    pub const fn from_c_ptr(addr: *mut c_void) -> Self {
        Address(addr as _)
    }

    #[inline(always)]
    pub const fn null() -> Self {
        Self::new(std::ptr::null())
    }

    #[inline(always)]
    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }

    #[inline(always)]
    pub fn is_not_null(&self) -> bool {
        !self.0.is_null()
    }

    #[inline(always)]
    pub fn uoffset(&self, size: usize) -> Address {
        self.offset(size as isize)
    }

    #[inline(always)]
    pub const fn offset(&self, size: isize) -> Address {
        unsafe { Address::new(self.0.offset(size)) }
    }

    #[inline(always)]
    pub const fn raw_ptr(&self) -> *const u8 {
        self.0
    }

    #[inline(always)]
    pub const fn as_mut_raw_ptr(&self) -> *mut u8 {
        self.0 as _
    }

    #[inline(always)]
    pub const fn deref_as_u8(&self) -> u8 {
        unsafe { *self.raw_ptr() }
    }

    #[inline(always)]
    pub fn as_usize(&self) -> usize {
        return self.0 as _;
    }

    #[inline(always)]
    pub fn as_isize(&self) -> isize {
        return self.0 as _;
    }
}

#[inline(always)]
pub const fn align(size: usize) -> usize {
    return align_of(size, POINTER_SIZE);
}

#[inline(always)]
const fn align_of(size: usize, align: usize) -> usize {
    debug_assert!(is_power_of_2(align));
    return (size + align - 1) & (!(align - 1));
}

#[inline(always)]
pub fn is_align_of(size: usize, align: usize) -> bool {
    (size & (align - 1)) == 0
}

#[inline(always)]
const fn is_power_of_2(val: usize) -> bool {
    (val & (val - 1)) == 0
}
