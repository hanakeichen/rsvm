use super::space::{SemiSpace, Space};
use super::Address;

pub struct GCStats {
    minor_gc_count: usize,
    minor_gc_time: usize,
    minor_gc_last_time: usize,
    major_gc_count: usize,
    major_gc_time: usize,
    major_gc_last_time: usize,
}

pub struct Heap {
    creation_space: Space,
    survivor_space: SemiSpace,
    old_space: Space,
    perm_space: Space,
    code_space: Space,
}

impl Heap {
    pub fn alloc_obj_permanent(&mut self, size: usize) -> Address {
        self.perm_space.alloc(size)
    }

    pub fn alloc_code(&mut self, size: usize) -> Address {
        self.code_space.alloc(size)
    }

    pub fn alloc_obj(&mut self, size: usize) -> Address {
        let mut result = self.alloc_obj_internal(size);
        if result.is_null() {
            self.minor_gc();
            result = self.alloc_obj_internal(size);
            if result.is_null() {
                // TODO
                panic!("out of memory");
            }
        }
        return result;
    }

    fn alloc_obj_internal(&mut self, size: usize) -> Address {
        self.creation_space.alloc(size)
    }

    fn minor_gc(&mut self) {
        // TODO
    }
}
