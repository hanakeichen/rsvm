use parking_lot::ReentrantMutex;

use super::parser::ClassParser;
use super::reader::{ClassReader, OwnedBytesClassReader};
use super::ClassLoadErr;
use crate::classfile::descriptor::{Descriptor, DescriptorParser};
use crate::object::hash_table::{GetEntryWithKey, HashTable, HashTablePtr};
use crate::object::prelude::*;
use crate::object::string::Utf8String;
use crate::thread::{Thread, ThreadPtr};
use crate::utils;
use std::cell::RefCell;
use std::fs::File;

#[derive(Default)]
pub struct BootstrapClassLoader {
    cp_entries: ReentrantMutex<RefCell<Vec<Box<dyn ClassPathEntry>>>>,
    loaded_classes: ReentrantMutex<RefCell<HashTablePtr>>,
}

impl BootstrapClassLoader {
    pub fn new(class_path: &str, current_dir: &str, thread: ThreadPtr) -> Self {
        let mut cp_entries: Vec<Box<dyn ClassPathEntry>> = Vec::new();

        if class_path.len() != 0 {
            let class_path_entries: Vec<&str> =
                class_path.split(utils::get_path_separator()).collect();
            for class_path_entry in class_path_entries {
                if class_path_entry == "." {
                    cp_entries.push(Box::new(ClassPathDirEntry::new(current_dir)));
                } else if class_path_entry.ends_with(".jar") {
                    if let Some(entry) = ClassPathJarEntry::with_jar(class_path_entry) {
                        cp_entries.push(Box::new(entry));
                    };
                } else {
                    cp_entries.push(Box::new(ClassPathDirEntry::new(class_path_entry)));
                }
            }
        }

        return Self {
            cp_entries: ReentrantMutex::new(RefCell::new(cp_entries)),
            loaded_classes: ReentrantMutex::new(RefCell::new(HashTable::new(thread))),
        };
    }

    pub(crate) fn add_preloaded_class(
        // self: &Arc<Self>,
        &self,
        cls: JClassPtr,
        thread: ThreadPtr,
    ) {
        let vm = thread.vm();
        assert!(vm.preloaded_classes().is_preloaded(cls));
        log::trace!(
            "class loader insert class {}, {:x}, getClass {:x}",
            cls.name().as_str(),
            cls.as_usize(),
            cls.jclass().as_usize()
        );
        self.do_with_mut_loaded_classes(|loaded_cls| {
            *loaded_cls = loaded_cls.insert(cls, thread);
        });
    }

    pub fn find_class(&self, class_name: &str) -> Option<JClassPtr> {
        return self.do_with_loaded_classes(|loaded_classes| {
            return loaded_classes.get_value_by_str(Utf8String::from(class_name));
        });
    }

    pub fn find_class_with_symbol(&self, class_name: SymbolPtr) -> Option<JClassPtr> {
        return self.do_with_loaded_classes(|loaded_classes| {
            return loaded_classes.get_value_by_str(class_name);
        });
    }

    pub fn load_binary_name_class(
        // self: &Arc<Self>,
        &self,
        binary_class_name: &str,
    ) -> Result<JClassPtr, ClassLoadErr> {
        let internal_class_name = binary_class_name.replace(".", "/");
        let thread = Thread::current();
        return self.load_class_depth(thread, internal_class_name.as_str(), 0);
    }

    pub fn load_class(
        // self: &Arc<Self>,
        &self,
        class_name: &str,
    ) -> Result<JClassPtr, ClassLoadErr> {
        let thread = Thread::current();
        return self.load_class_depth(thread, class_name, 0);
    }

    pub fn load_class_with_symbol(&self, class_name: SymbolPtr) -> Result<JClassPtr, ClassLoadErr> {
        let thread = Thread::current();
        return self.load_class_depth(thread, class_name.as_str(), 0);
    }

    pub(crate) fn load_class_depth(
        // self: &Arc<Self>,
        &self,
        thread: ThreadPtr,
        class_name: &str,
        depth: i32,
    ) -> Result<JClassPtr, ClassLoadErr> {
        assert!(class_name.len() > 0);
        if class_name.len() == 1 {
            if let Descriptor::ResolvedClass(class, _) =
                DescriptorParser::from_bytes(class_name.as_bytes(), thread.vm()).next()
            {
                if class.is_not_null() {
                    return Ok(class);
                }
            }
        }
        if let Some(find_cls) = self.find_class(class_name) {
            return Ok(find_cls);
        }
        let loaded_class = self.do_load_class(thread, class_name)?;
        // self.add_loaded_classes(&[loaded_class]);
        let _depth = depth + 1;
        // self.link_class(loaded_class, thread, depth)?;
        return Ok(loaded_class);
    }

    fn do_load_class(
        &self,
        thread: ThreadPtr,
        class_name: &str,
    ) -> Result<JClassPtr, ClassLoadErr> {
        assert!(class_name.len() > 0);
        if class_name.starts_with('[') && class_name.len() > 1 {
            let mut component_class_name = &class_name[1..];
            if component_class_name.starts_with('L') {
                component_class_name = &component_class_name[1..component_class_name.len() - 1];
            }
            let component_class = self.load_class(component_class_name)?;
            let class_name = thread.vm().symbol_table.get_or_insert(class_name);
            let result = JClass::new_array_class(class_name, component_class, Thread::current());
            self.do_with_mut_loaded_classes(|loaded_classes| {
                *loaded_classes = loaded_classes.insert(result, thread);
            });
            return Ok(result);
        }
        if class_name == "MethodCall$Sub" {
            println!("123");
        }
        let cp_entries = self.cp_entries.lock();
        for entry in unsafe { &mut *(*cp_entries).as_ptr() }.iter_mut() {
            if let Some(reader) = entry.reader(class_name) {
                return self.do_with_mut_loaded_classes(
                    |loaded_classes| -> Result<JClassPtr, ClassLoadErr> {
                        let mut parser =
                            ClassParser::new(thread.class_loader(), reader, thread.vm());
                        let result = parser.parse_class()?;
                        *loaded_classes = loaded_classes.insert(result, thread);
                        return Ok(result);
                    },
                );
            }
        }
        todo!(
            "throw ClassNotFoundException, cannot find class: {}",
            class_name
        );
    }

    fn do_with_loaded_classes<R, F: FnOnce(HashTablePtr) -> R>(&self, f: F) -> R {
        let loaded_classes = self.loaded_classes.lock();
        return f(unsafe { *(*loaded_classes).as_ptr() });
    }

    fn do_with_mut_loaded_classes<R, F: FnOnce(&mut HashTablePtr) -> R>(&self, f: F) -> R {
        let loaded_classes = self.loaded_classes.lock();
        return f(unsafe { &mut *(*loaded_classes).as_ptr() });
    }
}

impl GetEntryWithKey<SymbolPtr> for JClass {
    fn hash_key(ref_str: SymbolPtr) -> JInt {
        return ref_str.hash_code();
    }

    fn entry_equals_key(value: crate::memory::Address, ref_str: SymbolPtr) -> bool {
        let value = JClassPtr::from_addr(value);
        return value.name() == ref_str;
    }
}

const CLASS_SUFFIX: &'static str = ".class";
const CLASS_SUFFIX_LEN: usize = CLASS_SUFFIX.len();

trait ClassPathEntry {
    fn reader(&mut self, filename: &str) -> Option<Box<dyn ClassReader>>;
}

struct ClassPathDirEntry {
    dir: String,
}

impl ClassPathDirEntry {
    fn new(dir: &str) -> ClassPathDirEntry {
        let dir = if dir.ends_with("/") {
            dir.into()
        } else {
            let mut dir = String::from(dir);
            dir.push('/');
            dir
        };
        return ClassPathDirEntry { dir };
    }

    fn construct_full_path(&self, filename: &str) -> String {
        let mut full_path =
            String::with_capacity(self.dir.len() + filename.len() + CLASS_SUFFIX_LEN);
        full_path.push_str(&self.dir);
        full_path.push_str(&filename);
        full_path.push_str(".class");
        return full_path;
    }
}

impl ClassPathEntry for ClassPathDirEntry {
    fn reader(&mut self, filename: &str) -> Option<Box<dyn ClassReader>> {
        let full_path = self.construct_full_path(filename);
        let file_path = std::path::Path::new(&full_path);
        if let Ok(bytes) = std::fs::read(file_path) {
            log::trace!("find class success: {}", full_path);
            return Some(Box::new(OwnedBytesClassReader::new(bytes)));
        } else {
            return None;
        }
    }
}

struct ClassPathJarEntry {
    archive: zip::ZipArchive<File>,
}

impl ClassPathJarEntry {
    fn with_jar(jar: &str) -> Option<ClassPathJarEntry> {
        let archive = if let Ok(file) = File::open(jar) {
            if let Ok(archive) = zip::ZipArchive::new(file) {
                archive
            } else {
                return None;
            }
        } else {
            return None;
        };
        return Some(Self { archive });
    }

    fn construct_entry_path(filename: &str) -> String {
        let mut path = String::with_capacity(filename.len() + CLASS_SUFFIX_LEN);
        path.push_str(&filename);
        path.push_str(&CLASS_SUFFIX);
        return path;
    }
}

impl ClassPathEntry for ClassPathJarEntry {
    fn reader(&mut self, filename: &str) -> Option<Box<dyn ClassReader>> {
        let decrypt_start = std::time::SystemTime::now();
        let entry_name = Self::construct_entry_path(filename);
        return if let Ok(Ok(mut entry_file)) = self.archive.by_name_decrypt(&entry_name, &[]) {
            let mut buf = Vec::with_capacity(entry_file.size() as usize);
            // log::trace!("entry_file {} , size {}", entry_name, entry_file.size());
            if let Err(_) = std::io::copy(&mut entry_file, &mut buf) {
                return None;
            }
            {
                let cost = decrypt_start.elapsed().unwrap().as_millis();
                if cost > 1 * 100 {
                    log::info!(
                        "entry_file {} , size {}, costs {} seconds",
                        entry_name,
                        entry_file.size(),
                        cost
                    );
                }
            }
            debug_assert_eq!(buf.len(), entry_file.size() as usize);
            Some(Box::new(OwnedBytesClassReader::new(buf)))
        } else {
            None
        };
    }
}
