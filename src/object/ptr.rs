use crate::memory::Address;
use std::ops::{Deref, DerefMut};

pub struct Ptr<T> {
    ptr: *mut T,
}

impl<T> Ptr<T> {
    pub const fn new(ptr: *mut T) -> Ptr<T> {
        Ptr { ptr }
    }

    pub const fn from_addr(addr: Address) -> Ptr<T> {
        Ptr {
            ptr: addr.ptr() as _,
        }
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

    pub fn as_mut_ptr(&self) -> *mut T {
        self.ptr
    }

    pub fn as_address(&self) -> Address {
        Address::new(self.ptr.cast())
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
        unsafe { &mut *self.ptr }
    }
}

impl<T> Copy for Ptr<T> {}

impl<T> Clone for Ptr<T> {
    fn clone(&self) -> Ptr<T> {
        *self
    }
}
