#![feature(ptr_internals)]
#![feature(allocator_api)]
#![feature(alloc_layout_extra)]
#![feature(hash_set_entry)]
#![feature(get_mut_unchecked)]

pub mod classfile;
mod gc;
pub mod global;
mod handle;
mod interpreter;
mod memory;
mod object;
mod os;
mod thread;
mod vm;
