use super::ClassLoadErr;
use crate::handle::Handle;
use crate::object::prelude::*;

pub trait ClassLoader {
    fn load_class(&mut self, class_name: SymbolPtr) -> Result<Handle<Class>, ClassLoadErr>;

    fn find_class(&self, class_name: SymbolPtr) -> Result<Handle<Class>, ClassLoadErr>;

    fn resolve_class(&mut self, class_name: SymbolPtr) -> Result<Handle<Class>, ClassLoadErr>;
}

struct BootstrapClassloader;

impl BootstrapClassloader {
    fn load_class(class_name: &str) {}
}
