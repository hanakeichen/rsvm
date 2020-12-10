use crate::memory::Address;
use std::ops::{Deref, DerefMut};

pub struct Ptr<T> {
    ptr: *const T,
}

impl<T> Ptr<T> {
    pub const fn new(ptr: *const T) -> Ptr<T> {
        Ptr { ptr }
    }

    pub const fn from_addr(addr: Address) -> Ptr<T> {
        Ptr {
            ptr: addr.ptr() as _,
        }
    }

    pub const fn from_usize(addr: usize) -> Ptr<T> {
        Self::from_addr(Address::from_usize(addr))
    }

    pub const fn null() -> Ptr<T> {
        Ptr {
            ptr: std::ptr::null_mut(),
        }
    }

    pub fn cast<S>(&self) -> Ptr<S> {
        Ptr {
            ptr: self.ptr as *mut S,
        }
    }

    pub fn as_ptr(&self) -> *const T {
        self.ptr
    }

    pub fn as_mut_ptr(&self) -> *mut T {
        self.ptr as _
    }

    pub fn as_address(&self) -> Address {
        Address::new(self.ptr.cast())
    }

    pub fn as_usize(&self) -> usize {
        self.ptr as _
    }

    pub fn offset(&self, offset: isize) -> Ptr<T> {
        unsafe { Ptr::new(self.ptr.offset(offset)) }
    }
}

impl<T> Deref for Ptr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for Ptr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.ptr as *mut T) }
    }
}

impl<T> Copy for Ptr<T> {}

impl<T> Clone for Ptr<T> {
    fn clone(&self) -> Ptr<T> {
        *self
    }
}
