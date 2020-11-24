use crate::memory::Address;
use crate::object::prelude::*;
use crate::thread::Thread;
use std::mem;
use std::ops::{Deref, DerefMut};

type RawHandle = *mut Address;
type RawHandleArray = [Address; HANDLES_SIZE];

const HANDLE_PER_SIZE: usize = mem::size_of::<RawHandle>();
const HANDLES_SIZE: usize = HANDLE_PER_SIZE * 128;

pub struct HandleData {
    area: HandleArea,
    raw_handles: Vec<RawHandleArray>,
}

impl HandleData {
    fn new() -> Self {
        HandleData {
            area: HandleArea::new(),
            raw_handles: Vec::new(),
        }
    }

    fn new_area(&mut self) {
        self.area.size = 0;
    }

    fn handle_offset(&self) -> RawHandle {
        self.area.offset
    }
}

#[derive(Copy, Clone)]
struct HandleArea {
    offset: RawHandle,
    limit: RawHandle,
    size: usize,
}

impl HandleArea {
    fn new() -> Self {
        HandleArea {
            offset: std::ptr::null_mut(),
            limit: std::ptr::null_mut(),
            size: 0,
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
    pub fn new(thread: &mut Thread) -> HandleScope {
        let scope = HandleScope {
            prev: thread.handle_data().area,
        };
        thread.handle_data_mut().new_area();
        return scope;
    }

    fn make_handle(ptr: Address, thread: &mut Thread) -> RawHandle {
        let handle_data = thread.handle_data_mut();
        let handle = if !handle_data.area.available() {
            let mut new_handle = Self::alloc_handle();
            handle_data.raw_handles.push(new_handle);
            let new_handle = new_handle.as_mut_ptr();
            handle_data.area.offset = new_handle;
            handle_data.area.limit = unsafe { new_handle.offset(HANDLES_SIZE as isize) };
            handle_data.area.size += 1;
            new_handle
        } else {
            handle_data.area.offset
        };
        unsafe { *handle = ptr }
        handle_data.area.reserve(1);
        return handle;
    }

    fn alloc_handle() -> RawHandleArray {
        [Address::null(); HANDLES_SIZE]
    }
}

impl Drop for HandleScope {
    fn drop(&mut self) {
        let mut current_thread = Thread::current();
        let handle_data = current_thread.handle_data_mut();
        for _ in 0..handle_data.area.size {
            handle_data.raw_handles.pop();
        }
        handle_data.area = self.prev;
    }
}

pub struct Handle<T> {
    location: *mut *mut T,
}

impl<T> Handle<T> {
    pub fn new(ptr: Ptr<T>) -> Handle<T> {
        let current_thread = &mut Thread::current();
        Handle {
            location: HandleScope::make_handle(ptr.as_address(), current_thread) as *mut *mut T,
        }
    }

    pub fn value(&self) -> Ptr<T> {
        unsafe { Ptr::new(*self.location) }
    }

    pub fn as_ptr(&self) -> Ptr<T> {
        unsafe { Ptr::new(*self.location) }
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
