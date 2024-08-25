use crate::memory::Address;
use std::{
    ffi::c_void,
    ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub struct Ptr<T> {
    ptr: *const T,
}

impl<T> Ptr<T> {
    #[inline(always)]
    pub const fn new(ptr: *const T) -> Ptr<T> {
        Ptr { ptr }
    }

    #[inline(always)]
    pub const fn from_addr(addr: Address) -> Ptr<T> {
        Ptr {
            ptr: addr.raw_ptr() as _,
        }
    }

    #[inline(always)]
    pub const fn from_raw(raw_ptr: *const T) -> Ptr<T> {
        Ptr { ptr: raw_ptr }
    }

    #[inline(always)]
    pub const fn from_usize(addr: usize) -> Ptr<T> {
        Self::from_addr(Address::from_usize(addr))
    }

    #[inline(always)]
    pub const fn from_isize(addr: isize) -> Ptr<T> {
        Self::from_addr(Address::from_isize(addr))
    }

    #[inline(always)]
    pub const fn from_c_ptr(addr: *mut c_void) -> Ptr<T> {
        Self::from_addr(Address::from_c_ptr(addr))
    }

    // pub const fn from_ref(dst: &T) -> Ptr<T> {
    //     Ptr::new(dst as *const T)
    // }

    #[inline(always)]
    pub const fn from_self(dst: &T) -> Ptr<T> {
        Ptr::new(dst as *const T)
    }

    #[inline(always)]
    pub const fn from_ref<V>(dst: &V) -> Ptr<T> {
        Ptr::new(dst as *const V as *const T)
    }

    #[inline(always)]
    pub const fn from_ref_offset_bytes<V>(dst: &V, offset: isize) -> Ptr<T> {
        let ptr: Ptr<T> = Ptr::from_ref(dst);
        return ptr.cast::<u8>().offset(offset).cast::<T>();
    }

    #[inline(always)]
    pub const fn from_self_offset_bytes<R>(dst: &T, offset: isize) -> Ptr<R> {
        let ptr: Ptr<T> = Ptr::from_ref(dst);
        return ptr.cast::<u8>().offset(offset).cast::<R>();
    }

    #[inline(always)]
    pub const fn null() -> Ptr<T> {
        Ptr {
            ptr: std::ptr::null_mut(),
        }
    }

    #[inline(always)]
    pub fn is_null(&self) -> bool {
        return self.ptr.is_null();
    }

    #[inline(always)]
    pub fn is_not_null(&self) -> bool {
        return !self.ptr.is_null();
    }

    #[inline(always)]
    pub const fn cast<S>(&self) -> Ptr<S> {
        Ptr {
            ptr: self.ptr as *mut S,
        }
    }

    #[inline(always)]
    pub fn as_raw_ptr(&self) -> *const T {
        self.ptr
    }

    #[inline(always)]
    pub fn as_mut_raw_ptr(&self) -> *mut T {
        self.ptr as _
    }

    #[inline(always)]
    pub fn as_ref(&self) -> &T {
        unsafe { &*self.ptr }
    }

    #[inline(always)]
    pub fn as_mut_ref(&self) -> &mut T {
        unsafe { &mut *(self.ptr as *mut T) }
    }

    #[inline(always)]
    pub fn as_address(&self) -> Address {
        Address::new(self.ptr.cast())
    }

    #[inline(always)]
    pub fn as_usize(&self) -> usize {
        self.ptr as _
    }

    #[inline(always)]
    pub fn as_isize(&self) -> isize {
        self.ptr as _
    }

    #[inline(always)]
    pub fn as_c_ptr(&self) -> *mut c_void {
        self.ptr as _
    }

    #[inline(always)]
    pub fn as_slice<'a>(&'a self, len: usize) -> &'a [T] {
        unsafe { &*std::ptr::slice_from_raw_parts(self.ptr as _, len) }
    }

    #[inline(always)]
    pub fn as_mut_slice(&self, len: usize) -> &mut [T] {
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(self.ptr as _, len) }
    }

    #[inline(always)]
    pub const fn offset(&self, offset: isize) -> Ptr<T> {
        unsafe { Ptr::new(self.ptr.offset(offset)) }
    }
}

impl<T> Deref for Ptr<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for Ptr<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.ptr as *mut T) }
    }
}

impl<T> Copy for Ptr<T> {}

impl<T> Clone for Ptr<T> {
    #[inline(always)]
    fn clone(&self) -> Ptr<T> {
        Ptr::from_raw(self.ptr)
    }
}

impl<T> PartialEq for Ptr<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl<T> Default for Ptr<T> {
    #[inline(always)]
    fn default() -> Self {
        Self::null()
    }
}
