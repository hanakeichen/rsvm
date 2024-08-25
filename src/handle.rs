use crate::memory::Address;
use crate::object::prelude::*;
use crate::thread::{Thread, ThreadPtr};
use std::mem;
use std::ops::{Deref, DerefMut};

type RawHandle = *mut Address;
type RawHandleArray = Box<[Address]>;

const HANDLE_PER_SIZE: usize = mem::size_of::<RawHandle>();
const HANDLES_SIZE: usize = HANDLE_PER_SIZE * 128;

pub struct HandleData {
    area: HandleArea,
    raw_handles: Vec<RawHandleArray>,
}

impl HandleData {
    pub fn new() -> Self {
        HandleData {
            area: HandleArea::new(),
            raw_handles: Vec::new(),
        }
    }

    fn new_area(&mut self) {
        self.area.chunks = 0;
    }

    fn handle_offset(&self) -> RawHandle {
        self.area.offset
    }
}

#[derive(Copy, Clone)]
struct HandleArea {
    offset: RawHandle,
    limit: RawHandle,
    chunks: usize,
}

impl HandleArea {
    fn new() -> Self {
        HandleArea {
            offset: std::ptr::null_mut(),
            limit: std::ptr::null_mut(),
            chunks: 0,
        }
    }

    fn available(&self) -> bool {
        self.offset != self.limit
    }

    fn reserve(&mut self, size: usize) {
        unsafe {
            self.offset = self.offset.offset(size as isize);
        }
    }
}

pub struct HandleScope {
    prev: HandleArea,
}

impl HandleScope {
    pub fn new(thread: ThreadPtr) -> HandleScope {
        let scope = HandleScope {
            prev: thread.handle_data().area,
        };
        thread.as_mut_ref().handle_data_mut().new_area();
        return scope;
    }

    pub fn new_with_data(data: &mut HandleData) -> HandleScope {
        let scope = HandleScope { prev: data.area };
        data.new_area();
        return scope;
    }

    fn make_handle(ptr: Address, thread: ThreadPtr) -> RawHandle {
        let handle_data = thread.as_mut_ref().handle_data_mut();
        let handle = if !handle_data.area.available() {
            let mut new_handle = Self::alloc_handles();
            let new_handle_ptr = new_handle.as_mut_ptr();
            handle_data.raw_handles.push(new_handle);
            handle_data.area.offset = new_handle_ptr;
            handle_data.area.limit = unsafe { new_handle_ptr.offset(HANDLES_SIZE as isize) };
            handle_data.area.chunks += 1;
            new_handle_ptr
        } else {
            handle_data.area.offset
        };
        unsafe { *handle = ptr }
        handle_data.area.reserve(1);
        return handle;
    }

    fn alloc_handles() -> RawHandleArray {
        return vec![Address::null(); HANDLES_SIZE].into_boxed_slice();
    }
}

impl Drop for HandleScope {
    fn drop(&mut self) {
        let mut current_thread = Thread::current();
        let handle_data = current_thread.handle_data_mut();
        for _ in 0..handle_data.area.chunks {
            log::trace!("HandleScope::Drop pop");
            handle_data.raw_handles.pop();
        }
        handle_data.area = self.prev;
    }
}

#[derive(Clone, Copy)]
pub struct Handle<T> {
    location: *mut *mut T,
}

impl<T> Handle<T> {
    #[inline(always)]
    pub fn new(ptr: Ptr<T>) -> Handle<T> {
        let current_thread = Thread::current();
        Handle {
            location: HandleScope::make_handle(ptr.as_address(), current_thread) as *mut *mut T,
        }
    }

    #[inline(always)]
    pub fn new_with_thread(ptr: Ptr<T>, thread: ThreadPtr) -> Handle<T> {
        Handle {
            location: HandleScope::make_handle(ptr.as_address(), thread) as *mut *mut T,
        }
    }

    #[inline(always)]
    pub fn swap(&mut self, that: Handle<T>) {
        debug_assert!(self.is_not_null());
        debug_assert!(that.is_not_null());
        unsafe {
            let tmp = *self.location;
            *self.location = *that.location;
            *that.location = tmp;
        }
    }

    #[inline(always)]
    pub fn null() -> Handle<T> {
        Handle {
            location: std::ptr::null_mut(),
        }
    }

    #[inline(always)]
    pub fn value(&self) -> Ptr<T> {
        unsafe { Ptr::new(*self.location) }
    }

    #[inline(always)]
    pub fn set_value(&mut self, value: Ptr<T>) {
        unsafe {
            *self.location = value.as_mut_raw_ptr();
        }
    }

    pub fn as_ptr(&self) -> Ptr<T> {
        if self.location.is_null() {
            return Ptr::null();
        }
        unsafe { Ptr::from_raw(*self.location) }
    }

    #[inline(always)]
    pub fn get_ptr(&self) -> Ptr<T> {
        if self.location.is_null() {
            return Ptr::null();
        }
        unsafe { Ptr::from_raw(*self.location) }
    }

    #[inline(always)]
    pub fn is_null(&self) -> bool {
        return self.location.is_null();
    }

    #[inline(always)]
    pub fn is_not_null(&self) -> bool {
        return !self.location.is_null();
    }

    #[inline(always)]
    pub fn cast<R>(&self) -> Handle<R> {
        return Handle {
            location: self.location as _,
        };
    }
}

impl<T> Deref for Handle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &**self.location }
    }
}

impl<T> DerefMut for Handle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut **self.location }
    }
}
