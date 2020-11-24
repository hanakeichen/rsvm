use crate::memory::heap::Heap;
use crate::object::symbol::SymbolTable;

static mut VM_INSTANCE: *mut VM = std::ptr::null_mut();

pub fn instance() -> &'static mut VM {
    return unsafe { VM_INSTANCE.as_mut().expect("vm not initialized") };
}

pub struct VM {
    pub heap: Heap,
    // bootstrap_classes: BootstrapClassArray,
    pub symbol_table: SymbolTable,
}

impl VM {
    /* pub fn bootstrap_class(&self, classId: BootstrapClassId) -> ClassPtr {
        return ClassPtr::new(self.bootstrap_classes[classId] as Address);
    } */
}
