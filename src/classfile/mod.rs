pub mod class_loader;
pub mod parser;
pub mod reader;

pub use class_loader::ClassLoader;

pub type ClassLoadErr = &'static str;
