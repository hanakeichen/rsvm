use super::is_align_of;
use super::Address;
use crate::os;

pub struct Space {
    start: Address,
    end: Address,
    free: Address,
}

impl Space {
    pub fn new(start: Address, size: usize, exec: bool) -> Self {
        assert!(is_align_of(size, os::page_size()));
        if !os::commit_memory(start, size, exec) {
            panic!("could not commit memory");
        }
        return Space {
            start,
            end: start.offset(size as isize),
            free: start,
        };
    }

    pub fn alloc(&mut self, size: usize) -> Address {
        if self.free.uoffset(size) <= self.end {
            let result = self.free;
            self.free = self.free.offset(size as isize);
            return result;
        } else {
            return Address::null();
        }
    }

    pub fn reset(&mut self) {
        self.free = self.start;
    }
}

pub struct SemiSpace {
    from: Space,
    to: Space,
}

impl SemiSpace {
    pub fn new(start: Address, semi_size: usize) -> Self {
        SemiSpace {
            from: Space::new(start, semi_size, false),
            to: Space::new(start.uoffset(semi_size), semi_size, false),
        }
    }

    pub fn free(&self) -> Address {
        self.to.free
    }

    pub fn alloc(&mut self, size: usize) -> Address {
        self.to.alloc(size)
    }

    pub fn flip(&mut self) {
        std::mem::swap(&mut self.from, &mut self.to);
        self.to.reset();
    }
}
