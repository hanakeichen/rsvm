use super::space::{SemiSpace, Space};
use super::{Address, MB};
use crate::memory::space::SpaceType;
use crate::object::array::JArrayPtr;
use crate::object::class::{ClassData, JClass};
use crate::object::constant_pool::ConstantPoolPtr;
use crate::object::method::MethodPtr;
use crate::object::prelude::Ptr;
use crate::object::symbol::SymbolPtr;
use crate::object::Object;
use crate::thread::{Thread, ThreadPtr};
use crate::{os, JClassPtr, ObjectPtr};

pub type HeapPtr = Ptr<Heap>;

pub struct GCStats {
    minor_gc_count: usize,
    minor_gc_time: usize,
    minor_gc_last_time: usize,
    major_gc_count: usize,
    major_gc_time: usize,
    major_gc_last_time: usize,
}

pub struct Heap {
    new_space: SemiSpace,
    old_space: Space,
    perm_space: Space,
    code_space: Space,
    // lo_space: Space,
}

impl Heap {
    pub fn new() -> Heap {
        let survivor_space_size = 16 * MB;
        let old_space_size = 32 * MB;
        let perm_space_size = 8 * MB;
        let code_space_size = 8 * MB;
        // let lo_space_size = 32 * MB;

        let new_space =
            SemiSpace::new(os::reserve_memory(survivor_space_size), survivor_space_size);
        let old_space = Space::new(
            SpaceType::OLD,
            os::reserve_memory(old_space_size),
            old_space_size,
            false,
        );
        let perm_space = Space::new(
            SpaceType::PERM,
            os::reserve_memory(perm_space_size),
            perm_space_size,
            false,
        );
        let code_space = Space::new(
            SpaceType::CODE,
            os::reserve_memory(code_space_size),
            code_space_size,
            false,
        );

        return Heap {
            new_space,
            old_space,
            perm_space,
            code_space,
            // lo_space: Space::new(os::reserve_memory(lo_space_size), lo_space_size, false),
        };
    }

    pub fn debug(&self, prefix: &str) {
        log::debug!(
            "{} thread id {}, new_space: {:x?} {:x?} {:x?}, old_space: {:x?} {:x?}, perm_space: {:x?} {:x?}, code_space: {:x?} {:x?}",
            prefix,
            if Thread::current().is_not_null() { Thread::current().thread_id() } else { 999 },
            &self.new_space as *const SemiSpace,
            self.new_space.start(),
            self.new_space.end(),
            &self.old_space as *const Space,
            self.old_space.start(),
            &self.perm_space as *const Space,
            self.perm_space.start(),
            &self.code_space as *const Space,
            self.code_space.start()
        );
    }

    pub fn destroy(&self) {
        self.new_space.destroy();
        self.old_space.destroy();
        self.perm_space.destroy();
        self.code_space.destroy();
        // self.lo_space.destroy();
    }

    pub fn alloc_cls_permanent(
        &self,
        cp: ConstantPoolPtr,
        access_flags: u16,
        name: SymbolPtr,
        super_class: JClassPtr,
        interfaces: JArrayPtr,
        fields: JArrayPtr,
        methods: JArrayPtr,
        java_lang_class_inst_size: u16,
        static_fields_size: u16,
        vtab_len: u32,
        ifaces_len: u32,
        ifaces_m_indexes_len: u32,
        inst_size: u16,
        metadata_offset: u16,
        jclass_loader: ObjectPtr,
        init_method: MethodPtr,
        component_type: JClassPtr,
        thread: ThreadPtr,
    ) -> JClassPtr {
        debug_assert!(inst_size >= metadata_offset);
        let cls_size = JClass::size(
            java_lang_class_inst_size,
            static_fields_size,
            vtab_len,
            ifaces_len,
            ifaces_m_indexes_len,
        ) as usize;
        log::trace!(
            "JClass::new_permanent alloc_cls_permanent: {}, size {}",
            name.as_str(),
            cls_size
        );
        let jclass_addr = self.alloc_obj_permanent(cls_size);
        let jclass = JClassPtr::from_addr(jclass_addr);
        Object::init_header(jclass.cast(), thread.vm().preloaded_classes().jclass_cls());
        JClass::set_class_data(jclass, java_lang_class_inst_size);
        ClassData::parsed(
            jclass.class_data(),
            cp,
            access_flags,
            name,
            super_class,
            interfaces,
            fields,
            methods,
            jclass_loader,
            init_method,
            component_type,
            inst_size,
            metadata_offset,
            vtab_len,
            ifaces_len,
            ifaces_m_indexes_len,
        );
        return jclass;
    }

    pub fn alloc_obj_permanent(&self, size: usize) -> Address {
        assert!(super::is_align_of(size, super::POINTER_SIZE));
        return self.perm_space.alloc(size);
    }

    pub fn heap_contains(&self, addr: Address) -> bool {
        return self.new_contains(addr)
            || self.perm_contains(addr)
            || self.old_space.contains(addr)
            || self.code_space.contains(addr);
    }

    pub fn perm_contains(&self, addr: Address) -> bool {
        if !self.perm_space.contains(addr) {
            log::trace!(
                "perm_contains false {:x}, {:x}, {:x}",
                self.perm_space.start().as_usize(),
                self.perm_space.end().as_usize(),
                addr.as_usize()
            );
        }
        return self.perm_space.contains(addr);
    }

    pub fn new_contains(&self, addr: Address) -> bool {
        if !self.new_space.contains(addr) {
            log::trace!(
                "perm_contains false {:x}, {:x}, {:x}",
                self.new_space.start().as_usize(),
                self.new_space.end().as_usize(),
                addr.as_usize()
            );
        }
        return self.new_space.contains(addr);
    }

    pub fn alloc_code(&self, size: usize) -> Address {
        return self.code_space.alloc(size);
    }

    pub fn alloc_obj_lab(size: usize, thread: ThreadPtr) -> Address {
        let heap = thread.heap();
        let lab_capacity = thread.lab().capacity();
        if size > thread.lab().capacity() {
            return heap.new_space.alloc(size);
        }
        let result = Self::alloc_obj_lab_internal(size, thread);
        if result.is_not_null() {
            return result;
        }
        let buf = heap.new_space.alloc(lab_capacity);
        if buf.is_not_null() {
            let buf_limit = buf.uoffset(lab_capacity);
            thread.as_mut_ref().lab_mut().new_buf(buf, buf_limit);
            let result = Self::alloc_obj_lab_internal(size, thread);
            debug_assert!(result.is_not_null());
            return result;
        }
        return heap.alloc_obj(size);
    }

    fn alloc_obj_lab_internal(size: usize, thread: ThreadPtr) -> Address {
        let lab = thread.as_mut_ref().lab_mut();
        if size <= lab.available() {
            let result = lab.free();
            lab.set_free(result.uoffset(size));
            return result;
        }
        return Address::null();
    }

    fn alloc_obj(&self, size: usize) -> Address {
        assert!(super::is_align_of(size, super::POINTER_SIZE));
        let mut result = self.alloc_obj_internal(size);
        if result.is_null() {
            self.minor_gc();
            result = self.alloc_obj_internal(size);
            if result.is_null() {
                // TODO
                panic!("out of memory");
            }
        }
        return result;
    }

    fn alloc_obj_internal(&self, size: usize) -> Address {
        self.new_space.alloc(size)
    }

    fn minor_gc(&self) {
        // TODO
    }
}

impl Drop for Heap {
    fn drop(&mut self) {
        self.destroy();
    }
}
