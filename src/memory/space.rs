use std::sync::Mutex;

use super::is_align_of;
use super::Address;
use crate::os;

#[derive(Debug)]
pub enum SpaceType {
    NEW,
    OLD,
    PERM,
    CODE,
}

pub struct Space {
    space_type: SpaceType,
    start: Address,
    end: Address,
    free: Mutex<Address>,
}

impl Space {
    pub fn new(space_type: SpaceType, start: Address, size: usize, exec: bool) -> Self {
        assert!(is_align_of(size, os::page_size()));
        if !os::commit_memory(start, size, exec) {
            panic!("cannot commit memory");
        }
        return Space {
            space_type,
            start,
            end: start.offset(size as isize),
            free: Mutex::new(start),
        };
    }

    pub fn destroy(&self) {
        let ret = os::release_memory(self.start, self.size());
        if ret != 0 {
            panic!("release memory failed with error code {}", ret);
        }
    }

    pub fn alloc(&self, size: usize) -> Address {
        let mut free = self.free.lock().expect("Space::alloc failed");
        if free.uoffset(size) <= self.end {
            let result = *free;
            *free = result.offset(size as isize);
            unsafe {
                libc::memset(result.raw_ptr() as _, 0, size);
            }
            debug_assert!(result.offset(size as isize).as_usize() <= self.end.as_usize());
            debug_assert!(result.as_usize() >= self.start.as_usize());
            return result;
        } else {
            log::trace!("space {:?} overflow", self.space_type);
            return Address::null();
        }
    }

    pub fn contains(&self, addr: Address) -> bool {
        return addr.as_usize() >= self.start.as_usize() && addr.as_usize() < self.end.as_usize();
    }

    pub fn start(&self) -> Address {
        self.start
    }

    pub fn end(&self) -> Address {
        self.end
    }

    pub fn size(&self) -> usize {
        return self.end.as_usize() - self.start.as_usize();
    }

    pub fn reset(&self) {
        let mut free = self.free.lock().expect("Space::reset failed");
        *free = self.start;
    }
}

pub struct SemiSpace {
    start: Address,
    from: Space,
    to: Space,
}

impl SemiSpace {
    pub fn new(start: Address, size: usize) -> Self {
        let semi_size = size / 2;
        SemiSpace {
            start,
            from: Space::new(SpaceType::NEW, start, semi_size, false),
            to: Space::new(SpaceType::NEW, start.uoffset(semi_size), semi_size, false),
        }
    }

    pub fn destroy(&self) {
        let ret = os::release_memory(self.start, self.from.size() * 2);
        if ret != 0 {
            panic!("release SemiSpace failed with error code {}", ret);
        }
    }

    pub fn contains(&self, addr: Address) -> bool {
        return self.to.contains(addr);
    }

    pub fn start(&self) -> Address {
        self.to.start
    }

    pub fn end(&self) -> Address {
        self.to.end
    }

    pub fn alloc(&self, size: usize) -> Address {
        self.to.alloc(size)
    }

    pub fn flip(&mut self) {
        std::mem::swap(&mut self.from, &mut self.to);
        self.to.reset();
    }
}
