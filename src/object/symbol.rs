// use super::{ClassPtr, Header, ObjectTag, Ptr};
// use crate::global::Address;
use super::ptr::Ptr;
use std::collections::HashSet;

// static mut SYMBOL_TABLE: HashSet<String> = HashSet::new();

pub type SymbolPtr = Ptr<String>;

pub struct SymbolTable(HashSet<String>);

impl SymbolTable {
    fn new() -> SymbolTable {
        SymbolTable { 0: HashSet::new() }
    }

    pub fn get_or_insert(&mut self, content: String) -> SymbolPtr {
        SymbolPtr::new(self.0.get_or_insert(content))
    }
}

// pub struct Symbol {
//     content: &'static str,
// }

// impl Symbol {
//     pub const Symbol_CLASS: ClassPtr = ClassPtr::new(ObjectTag::Symbol as u8 as Address);

//     fn new(length: u16, bytes: *const u8) -> SymbolPtr {
//         let size = Header::size() + size_of::<u8>() * length as usize;
//         let symbol = SymbolPtr::new(vm::global().heap.alloc(size));
//         symbol.header.initialize(Symbol::Symbol_CLASS);
//         symbol.bytes = bytes;
//         return symbol;
//     }
// }
// pub fn new(content: String) -> SymbolPtr {
//     SYMBOL_TABLE.get_or_insert(content) as SymbolPtr
// }
