use super::parser::ClassParser;
use super::reader::{ClassReader, OwnedBytesClassReader};
use super::ClassLoadErr;
use crate::handle::Handle;
use crate::object::prelude::*;

use std::collections::HashMap;

trait ClassPathEntry {
    fn reader(&self, filename: &str) -> Option<Box<dyn ClassReader>>;
}

struct ClassPathDirEntry {
    dir: String,
}

impl ClassPathEntry for ClassPathDirEntry {
    fn reader(&self, filename: &str) -> Option<Box<dyn ClassReader>> {
        let class_file = String::with_capacity(self.dir.len() + filename.len());
        if let Ok(bytes) = std::fs::read(class_file) {
            return Some(Box::new(OwnedBytesClassReader::new(bytes)));
        } else {
            return None;
        }
    }
}

pub trait ClassLoader {
    fn resolve_class(&mut self, class_name: &str) -> Result<Handle<Class>, ClassLoadErr>;
}

struct NativeClassLoader {
    cp_entries: Vec<Box<dyn ClassPathEntry>>,
    loaded_classes: HashMap<String, ClassPtr>,
}

impl NativeClassLoader {
    fn load_class(&mut self, class_name: &str) -> Result<Handle<Class>, ClassLoadErr> {
        for entry in self.cp_entries.iter_mut() {
            if let Some(reader) = entry.reader(class_name) {
                let mut parser = ClassParser::new(self, reader);
                return parser.parse_class();
            }
        }
        panic!("cannot find class: {}", class_name); // TODO: ClassNotFoundException
    }
}

impl ClassLoader for NativeClassLoader {
    fn resolve_class(&mut self, class_name: &str) -> Result<Handle<Class>, ClassLoadErr> {
        if let Some(class) = self.loaded_classes.get(class_name) {
            return Ok(Handle::new(*class));
        }
        // TODO: mutex
        return Ok(self.load_class(class_name)?);
    }
}

pub struct BootstrapClassLoader {
    cl_impl: NativeClassLoader,
}

impl ClassLoader for BootstrapClassLoader {
    fn resolve_class(&mut self, class_name: &str) -> Result<Handle<Class>, ClassLoadErr> {
        self.cl_impl.resolve_class(class_name)
    }
}
