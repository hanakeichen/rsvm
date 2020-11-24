pub mod array;
pub mod class;
pub mod prelude;
mod ptr;
pub mod symbol;

use self::class::ClassPtr;
use crate::memory::Address;

use std::mem::size_of;

struct Header {
    forward_addr: Address,
    class: ClassPtr,
}

impl Header {
    fn initialize(&mut self, class: ClassPtr) {
        self.forward_addr = Address::null();
        self.class = class;
    }

    pub fn size() -> usize {
        size_of::<Header>()
    }
}

enum ObjectTag {
    Field = 0x01,
    Method = 0x02,
    ConstantPool = 0x03,
}

pub struct Object {
    header: Header,
    // data: *mut u8,
}

impl Object {
    pub fn initialize(&mut self, class: ClassPtr) {
        self.header.initialize(class)
    }

    pub fn class(&self) -> ClassPtr {
        return self.header.class;
    }

    fn is_jobject(&self) -> bool {
        false
    }
}
