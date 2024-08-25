pub mod class_info;
pub mod class_loader;
pub mod parser;
pub mod reader;
pub mod descriptor;

// pub use class_loader::ClassLoader;

// pub type ClassLoadErr = String;

#[derive(Debug)]
pub enum ClassLoadErr {
    InvalidFormat(String),
    VerifyFailed(String),
    ClassLoaderInvalidLockState(String),
}
