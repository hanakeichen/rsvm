#![feature(hash_set_entry)]
#![feature(get_mut_unchecked)]
#![feature(new_uninit)]
#![feature(slice_as_chunks)]
#![feature(thread_id_value)]
#![feature(atomic_from_ptr)]
#![feature(strict_provenance)]

pub use object::prelude::{JArray, JClassPtr, ObjectPtr};

pub mod classfile;
mod gc;
mod handle;
mod memory;
mod native;
mod object;
mod os;
mod runtime;
mod shared;
pub mod thread;
mod utils;
pub mod value;
pub mod vm;

#[cfg(any(test, feature = "rsvm_test"))]
mod test;
