use super::{Address, KB};

pub struct LocalAllocBuf {
    free: Address,
    limit: Address,
    capacity: usize,
}

impl LocalAllocBuf {
    pub fn new(free: Address, limit: Address) -> Self {
        return Self { free, limit, capacity: 1 * KB };
    }

    pub fn free(&self) -> Address {
        self.free
    }

    pub fn set_free(&mut self, free: Address) {
        self.free = free;
    }

    pub fn new_buf(&mut self, free: Address, limit: Address) {
        self.free = free;
        self.limit = limit;
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn available(&self) -> usize {
        return self.limit.as_usize() - self.free.as_usize();
    }
}

impl Default for LocalAllocBuf {
    fn default() -> Self {
        Self::new(Address::null(), Address::null())
    }
}
