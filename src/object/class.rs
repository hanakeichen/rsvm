use super::array::JArrayPtr;
use super::constant_pool::{ConstMemberRef, ConstantPoolPtr};
use super::field::FieldPtr;
use super::hash_table::GetEntryWithKey;
use super::method::{MethodIndex, MethodPtr, ResolvedMethod};
use super::ptr::Ptr;
use super::string::Utf8String;
use super::symbol::Symbol;
use super::{prelude::*, VMObject};
use crate::classfile::ClassLoadErr;
use crate::define_oop;
use crate::memory::{align, Address};
use crate::thread::{Thread, ThreadPtr};
use crate::vm::{VMPtr, VM};
use core::str;
use std::convert::From;
use std::mem::size_of;

pub type VTablePtr = Ptr<VTable>;
pub type ClassDataPtr = Ptr<ClassData>;
pub type JClassPtr = Ptr<JClass>;

type MethodCArray = Ptr<MethodPtr>;
type InterfaceCArray = Ptr<JClassPtr>;

#[allow(unused)]
#[repr(u16)]
pub enum ClassAccessFlags {
    AccPublic = 0x0001,
    AccFinal = 0x0010,
    AccSuper = 0x0020,
    AccInterface = 0x0200,
    AccAbstract = 0x0400,
    AccSynthetic = 0x1000,
    AccAnnotation = 0x2000,
    AccEnum = 0x4000,
}

impl ClassAccessFlags {
    #[inline(always)]
    pub const fn is_interface(access_flags: u16) -> bool {
        access_flags & ClassAccessFlags::AccInterface as u16
            == ClassAccessFlags::AccInterface as u16
    }
}

#[derive(Default)]
pub struct VTableInfo {
    // vtab_len: u32,
    // ifaces_len: u32,
    // ifaces_methods_len: u32,
    methods: Vec<MethodPtr>,
    ifaces: Vec<JClassPtr>,
    ifaces_m_indexes: Vec<IMethodIndex>,
}

/// VTable layout
///  --------------------------------
/// |            vtab_len            |
///  --------------------------------
/// |          interfaces_len        |
///  --------------------------------
/// |       ifaces_methods_len       |
///  --------------------------------
/// |             methods            |
///  --------------------------------
/// |            interfaces          |  \
///  --------------------------------      itable
/// |  method-indexes of interfaces  |  /
///  --------------------------------
#[derive(Debug)]
pub struct VTable {
    vtab_len: u32,
    ifaces_len: u32,
    ifaces_methods_len: u32,
}

type IMethodIndex = u32;
type IMethodIndexCArray = Ptr<IMethodIndex>;

impl VTable {
    const METHODS_OFFSET: usize = align(std::mem::size_of::<VTable>());

    pub fn obtain_vtab_info(
        access_flags: u16,
        methods: JArrayPtr,
        super_class: JClassPtr,
        interfaces: JArrayPtr,
        ctor_init_name: SymbolPtr,
        // ifaces_len: &mut u32,
        // ifaces_methods_len: &mut u32,
    ) -> VTableInfo {
        debug_assert!(!ClassAccessFlags::is_interface(access_flags));
        let mut info = VTableInfo::default();
        // let mut super_vtab_methods = MethodCArray::null();
        // let mut super_vtab_len: u32 = 0;
        if super_class.is_not_null() {
            let super_vtab = super_class.class_data().vtab();
            //let super_vtab_methods = super_class.class_data().vtab().methods();
            // let mut vtab_len = super_vtab_len;
            info.methods = super_vtab
                .methods()
                .as_slice(super_vtab.vtab_len as usize)
                .to_vec();
            let methods_len = methods.length();
            for m_idx in 0..methods_len {
                let method: MethodPtr = methods.get(m_idx).cast();
                if Self::method_is_not_vtab_member(method, ctor_init_name) {
                    continue;
                }
                let method_override_idx = Self::get_method_override_idx(method, &info.methods);
                if method_override_idx != -1 {
                    info.methods[method_override_idx as usize] = method;
                    continue;
                }
                log::trace!("info.methods.push {}", method.name().as_str());
                info.methods.push(method);
                // vtab_len += 1;
            }
            // vtab_len
        } else {
            let methods_len = methods.length();
            for m_idx in 0..methods_len {
                let method: MethodPtr = methods.get(m_idx).cast();
                if Self::method_is_not_vtab_member(method, ctor_init_name) {
                    continue;
                }
                info.methods.push(method);
            }
        }
        // TODO interface super class
        if interfaces.is_not_null() && interfaces.length() > 0 {
            Self::obtain_itable(interfaces, &mut info);
        }
        return info;
    }

    fn obtain_itable(interfaces: JArrayPtr, info: &mut VTableInfo) {
        let ifs_len = interfaces.length();
        for if_idx in 0..ifs_len {
            let iface: JClassPtr = interfaces.get(if_idx).cast();
            debug_assert!(iface.class_data().is_interface());
            // info.ifaces.push(iface);
            let mut iface = iface;
            loop {
                info.ifaces.push(iface);
                let iface_methods = iface.class_data().methods;
                let iface_methods_len = iface_methods.length();
                // *ifaces_methods_len += iface_methods_len as u32;
                for iface_m_idx in 0..iface_methods_len {
                    let iface_method: MethodPtr = iface_methods.get(iface_m_idx).cast();
                    let impl_idx = Self::find_method(
                        iface_method,
                        MethodCArray::from_raw(info.methods.as_ptr()),
                        info.methods.len() as JInt,
                    );
                    if impl_idx != -1 {
                        info.ifaces_m_indexes.push(impl_idx as IMethodIndex);
                        continue;
                    }
                    info.ifaces_m_indexes
                        .push(info.methods.len() as IMethodIndex);
                    // vtab_len += iface.jclass().class_data().methods.length() as u32;
                    // *vtab_len += 1;
                    info.methods.push(iface_method);
                }
                log::trace!("obtain_interfaces_indexes iface {:x}", iface.as_usize(),);
                log::trace!(
                    "obtain_interfaces_indexes iface {:x}, iface name {}, iface super_class: 0x{:x}",
                    iface.as_usize(),
                    iface.name().as_str(),
                    iface.class_data().super_class().as_usize()
                );
                iface = iface.class_data().super_class();
                if iface.is_null() || !iface.class_data().is_interface() {
                    break;
                }
                // *ifaces_len += 1;
            }
        }
    }

    fn initialize(
        &self,
        super_class: JClassPtr,
        methods: JArrayPtr,
        interfaces: JArrayPtr,
        vm: &VM,
    ) {
        let ctor_init_name = vm.shared_objs().symbols().ctor_init;
        let vtab = VTablePtr::from_ref(self);
        let vtab_methods = vtab.methods();
        let mut vtab_offset;
        if super_class.is_not_null() {
            let super_vtab = super_class.class_data().vtab();
            let super_vtab_methods = super_vtab.methods();
            let super_vtab_len = super_vtab.vtab_len;
            unsafe {
                std::ptr::copy(
                    super_vtab_methods.as_mut_raw_ptr(),
                    vtab_methods.as_mut_raw_ptr(),
                    super_vtab_len as usize,
                );
            }
            vtab_offset = super_vtab_len;
            for m_idx in 0..methods.length() {
                let method: MethodPtr = methods.get(m_idx).cast();
                if Self::method_is_not_vtab_member(method, ctor_init_name) {
                    continue;
                }
                let overridden_m_idx =
                    VTable::find_method(method, super_vtab_methods, super_vtab_len as JInt);
                if overridden_m_idx != -1 {
                    *vtab_methods.offset(overridden_m_idx as isize) = method;
                } else {
                    *vtab_methods.offset(vtab_offset as isize) = method;
                    vtab_offset += 1;
                }
            }
        } else {
            vtab_offset = 0;
            for m_idx in 0..methods.length() {
                let method: MethodPtr = methods.get(m_idx).cast();
                if Self::method_is_not_vtab_member(method, ctor_init_name) {
                    continue;
                }
                *vtab_methods.offset(vtab_offset as isize) = method;
                vtab_offset += 1;
            }
        }
        if interfaces.is_not_null() && interfaces.length() > 0 {
            Self::init_itable(vtab, interfaces, vtab_methods, &mut vtab_offset);
        }
    }

    fn init_itable(
        vtab: VTablePtr,
        interfaces: JArrayPtr,
        vtab_methods: MethodCArray,
        // iface: JClassPtr,
        vtab_offset: &mut u32,
    ) {
        let vtab_ifaces = vtab.ifaces();
        let mut vtab_ifaces_offset: u32 = 0;
        let imethod_indexes = vtab.imethod_indexes();
        let mut imethod_offset: u32 = 0;
        for iface_idx in 0..interfaces.length() {
            let mut iface: JClassPtr = interfaces.get(iface_idx).cast();
            loop {
                *vtab_ifaces.offset(vtab_ifaces_offset as isize) = iface;
                vtab_ifaces_offset += 1;
                let iface_methods = iface.class_data().methods;
                for iface_m_idx in 0..iface_methods.length() {
                    let iface_m: MethodPtr = iface_methods.get(iface_m_idx).cast();
                    if *vtab_offset > 0 {
                        let override_idx =
                            VTable::find_method(iface_m, vtab_methods, *vtab_offset as JInt);
                        let imethod_idx = if override_idx != -1 {
                            override_idx as IMethodIndex
                        } else {
                            let imethod_idx = *vtab_offset;
                            *vtab_offset = imethod_idx + 1;
                            imethod_idx as IMethodIndex
                        };
                        *imethod_indexes.offset(imethod_offset as isize) = imethod_idx;
                        imethod_offset += 1;
                    } else {
                        *vtab_methods.as_mut_ref() = iface_m;
                        *imethod_indexes.as_mut_ref() = 0;
                        *vtab_offset += 1;
                        imethod_offset += 1;
                    }
                }
                let super_class = iface.class_data().super_class();
                if super_class.is_null() || !super_class.class_data().is_interface() {
                    break;
                }
                iface = super_class;
            }
        }
        debug_assert_eq!(vtab.ifaces_len, vtab_ifaces_offset);
        debug_assert_eq!(vtab.ifaces_methods_len, imethod_offset);
    }

    fn get_method_override_idx(current_method: MethodPtr, methods: &Vec<MethodPtr>) -> JInt {
        if current_method.is_private() {
            return -1;
        }
        for m_idx in 0..methods.len() {
            let method = unsafe { methods.get_unchecked(m_idx) };
            if current_method.name() == method.name()
                && current_method.descriptor() == method.descriptor()
            {
                // log::trace!("vtab method_is_override current name {}, addr 0x{:x}, current descriptor {}, addr 0x{:x}, it_method name {}, addr 0x{:x}, it_method descriptor {}, addr 0x{:x}, result {}",
                //         current_method.name().as_str(), current_method.name().as_isize(),
                //         current_method.descriptor().as_str(), current_method.descriptor().as_isize(),
                //         method.name().as_str(), method.name().as_isize(),
                //         method.descriptor().as_str(), method.descriptor().as_isize(),
                //         m_idx as JInt
                //     );
                return m_idx as JInt;
            }
        }
        // log::trace!("vtab method_is_override current name {}, addr 0x{:x}, current descriptor {}, addr 0x{:x}, result {}",
        //                 current_method.name().as_str(), current_method.name().as_isize(),
        //                 current_method.descriptor().as_str(), current_method.descriptor().as_isize(),
        //                 -1
        //             );
        return -1;
    }

    fn find_method(current_method: MethodPtr, methods: MethodCArray, methods_len: JInt) -> JInt {
        for m_idx in 0..methods_len {
            let method = *methods.offset(m_idx as isize);
            // log::trace!("vtab find_method current name {}, addr 0x{:x}, current descriptor {}, addr 0x{:x}, it_method name {}, addr 0x{:x}, it_method descriptor {}, addr 0x{:x}",
            //     current_method.name().as_str(), current_method.name().as_isize(),
            //     current_method.descriptor().as_str(), current_method.descriptor().as_isize(),
            //     method.name().as_str(), method.name().as_isize(),
            //     method.descriptor().as_str(), method.descriptor().as_isize(),
            // );
            if current_method.name() == method.name()
                && current_method.descriptor() == method.descriptor()
            {
                return m_idx;
            }
        }
        return -1;
    }

    fn methods(&self) -> MethodCArray {
        return MethodCArray::from_addr(
            Address::from_ref(self).offset(Self::METHODS_OFFSET as isize),
        );
    }

    fn ifaces_len(&self) -> u32 {
        self.ifaces_len
    }

    fn ifaces(&self) -> InterfaceCArray {
        return InterfaceCArray::from_addr(
            Address::from_ref(self)
                .offset(Self::METHODS_OFFSET as isize)
                .offset(self.vtab_len as isize * std::mem::size_of::<MethodPtr>() as isize),
        );
    }

    fn imethod_indexes(&self) -> IMethodIndexCArray {
        return IMethodIndexCArray::from_addr(
            Address::from_ref(self)
                .offset(Self::METHODS_OFFSET as isize)
                .offset(self.vtab_len as isize * std::mem::size_of::<MethodPtr>() as isize)
                .offset(self.ifaces_len as isize * std::mem::size_of::<JClassPtr>() as isize),
        );
    }

    fn method_is_not_vtab_member(method: MethodPtr, ctor_init_name: SymbolPtr) -> bool {
        return method.is_private() || method.is_static() || ctor_init_name == method.name();
    }

    const fn size(vtab_len: u32, ifaces_len: u32, ifaces_m_indexes_len: u32) -> usize {
        return (Self::METHODS_OFFSET + std::mem::size_of::<MethodPtr>() * vtab_len as usize)
            + Self::itable_size(ifaces_len, ifaces_m_indexes_len);
    }

    const fn itable_size(ifaces_len: u32, ifaces_m_indexes_len: u32) -> usize {
        return std::mem::size_of::<JClassPtr>() * ifaces_len as usize
            + std::mem::size_of::<IMethodIndex>() * ifaces_m_indexes_len as usize;
    }
}

#[derive(Debug)]
pub struct ClassData {
    pub cp: ConstantPoolPtr,
    name: SymbolPtr,
    super_class: JClassPtr,
    interfaces: JArrayPtr,
    fields: JArrayPtr,
    methods: JArrayPtr,
    inners: JArrayPtr,
    jclass_loader: ObjectPtr,
    init_method: MethodPtr,
    component_type: JClassPtr,
    inst_or_ele_size: u16,
    metadata_offset: u16,
    access_flags: u16,
    is_primitive: bool,
    is_array: bool,
    _vtab: VTablePtr,
}

impl ClassData {
    // pub fn new_permanent(
    //     cp: ConstantPoolPtr,
    //     access_flags: u16,
    //     name: SymbolPtr,
    //     super_class: ClassPtr,
    //     interfaces: JArrayPtr,
    //     fields: JArrayPtr,
    //     methods: MethodArrayPtr,
    //     jclass_loader: ObjectPtr,
    //     jclass_jclass: ClassPtr,
    //     thread: ThreadPtr,
    // ) -> ClassPtr {
    //     let class = Self::new_unparsed(thread);
    //     Self::parsed(
    //         class,
    //         cp,
    //         access_flags,
    //         name,
    //         super_class,
    //         interfaces,
    //         fields,
    //         methods,
    //         jclass_loader,
    //         jclass_jclass,
    //     );
    //     return class;
    // }

    // pub fn new_unparsed(thread: ThreadPtr) -> ClassPtr {
    //     return ClassPtr::from_addr(thread.heap().alloc_obj_permanent(size_of::<Class>()));
    // }

    pub fn parsed(
        mut class_data: ClassDataPtr,
        cp: ConstantPoolPtr,
        access_flags: u16,
        name: SymbolPtr,
        super_class: JClassPtr,
        interfaces: JArrayPtr,
        fields: JArrayPtr,
        methods: JArrayPtr,
        jclass_loader: ObjectPtr,
        init_method: MethodPtr,
        component_type: JClassPtr,
        inst_or_ele_size: u16,
        metadata_offset: u16,
        vtab_len: u32,
        ifaces_len: u32,
        ifaces_methods_len: u32,
    ) {
        // let class_data = class.as_mut_ref();
        class_data.access_flags = access_flags;
        class_data.cp = cp;
        class_data.name = name;
        class_data.super_class = super_class;
        class_data.interfaces = interfaces;
        class_data.fields = fields;
        class_data.methods = methods;
        class_data.jclass_loader = jclass_loader;
        class_data.init_method = init_method;
        class_data.component_type = component_type;
        class_data.inst_or_ele_size = inst_or_ele_size;
        class_data.metadata_offset = metadata_offset;
        class_data._vtab = Self::vtab_slow(class_data);

        let mut vtab = class_data.vtab();
        vtab.vtab_len = vtab_len;
        vtab.ifaces_len = ifaces_len;
        vtab.ifaces_methods_len = ifaces_methods_len;
    }

    // pub fn new_primitive_class(
    //     class_name: SymbolPtr,
    //     instance_size: usize,
    //     thread: ThreadPtr,
    // ) -> ClassPtr {
    //     // let class_name = vm::instance().symbol_table.get_or_insert(name);
    //     // debug_assert_eq!(class_name.as_str(), name);
    //     let access_flags = ClassAccessFlags::AccPublic as u16 | ClassAccessFlags::AccFinal as u16;
    //     let mut class = ClassPtr::from_addr(thread.heap().alloc_obj_permanent(size_of::<Class>()));
    //     class.access_flags = access_flags;
    //     class.name = class_name;
    //     class.ins_or_ele_size = instance_size as u16;
    //     class.is_primitive = true;
    //     log::trace!(
    //         "new_primitive_class {}, cls addr {:x}, name addr {:x}",
    //         class_name.as_str(),
    //         class.as_usize(),
    //         class_name.as_usize()
    //     );
    //     return class;
    // }

    // pub fn new_vm_internal_class(
    //     class_name: SymbolPtr,
    //     instance_size: usize,
    //     thread: ThreadPtr,
    // ) -> ClassPtr {
    //     // let class_name = vm::instance().symbol_table.get_or_insert(name);
    //     // debug_assert_eq!(class_name.as_str(), name);
    //     let access_flags = ClassAccessFlags::AccPublic as u16 | ClassAccessFlags::AccFinal as u16;
    //     let mut class = ClassPtr::from_addr(thread.heap().alloc_obj_permanent(size_of::<Class>()));
    //     class.access_flags = access_flags;
    //     class.name = class_name;
    //     class.ins_or_ele_size = instance_size as u16;
    //     class.is_primitive = false;
    //     log::trace!(
    //         "new_vm_internal_class {}, cls addr {:x}, name addr {:x}",
    //         class_name.as_str(),
    //         class.as_usize(),
    //         class_name.as_usize()
    //     );
    //     return class;
    // }

    // pub fn new_primitive_array_class(
    //     name: SymbolPtr,
    //     element_size: usize,
    //     thread: ThreadPtr,
    // ) -> ClassPtr {
    //     return Self::new_array_class(name, element_size, thread);
    // }

    // pub fn new_reference_array_class(name: SymbolPtr, thread: ThreadPtr) -> ClassPtr {
    //     return Self::new_array_class(name, size_of::<Address>(), thread);
    // }

    pub fn descriptor_to_class_name(name: &str) -> &str {
        if name.starts_with('L') {
            return &name[1..name.len() - 1];
        }
        return name;
    }

    /// class preparation
    // pub fn prepare(&mut self, heap: &Heap) {
    //     self.setup_fields_layout(heap);
    // }

    pub fn is_interface(&self) -> bool {
        ClassAccessFlags::is_interface(self.access_flags)
    }

    pub fn is_abstract(&self) -> bool {
        self.access_flags & ClassAccessFlags::AccAbstract as u16
            == ClassAccessFlags::AccAbstract as u16
    }

    pub fn is_acc_super(&self) -> bool {
        self.access_flags & ClassAccessFlags::AccSuper as u16 == ClassAccessFlags::AccSuper as u16
    }

    pub fn name(&self) -> SymbolPtr {
        // Thread::current().heap().debug();
        if !Thread::current()
            .heap()
            .perm_contains(self.name.as_address())
        {
            Thread::current().heap().debug("Class::name !contains");
            log::trace!(
                "Class::name self {:x} name {:x}",
                ClassDataPtr::from_ref(self).as_usize(),
                self.name.as_usize(),
            );
        }
        debug_assert!(Thread::current()
            .heap()
            .perm_contains(self.name.as_address()));
        self.name
    }

    pub fn inst_or_ele_size(&self) -> usize {
        // if self.instance_size.low != 0 {
        //     return self.instance_size.low as usize;
        // }
        return self.inst_or_ele_size as usize;
    }

    pub fn metadata_offset(&self) -> u16 {
        self.metadata_offset
    }

    pub fn super_class(&self) -> JClassPtr {
        return self.super_class;
    }

    // pub fn set_super_class(&mut self, class: ClassPtr) {
    //     self.super_class = class;
    // }

    pub fn interfaces(&self) -> JArrayPtr {
        self.interfaces
    }

    pub fn fields(&self) -> JArrayPtr {
        self.fields
    }

    pub fn methods(&self) -> JArrayPtr {
        self.methods
    }

    pub fn inners(&self) -> JArrayPtr {
        self.inners
    }

    pub fn set_inners(&mut self, inners: JArrayPtr) {
        self.inners = inners;
    }

    pub fn component_type(&self) -> JClassPtr {
        self.component_type
    }

    pub fn access_flags(&self) -> u16 {
        self.access_flags
    }

    // pub fn get_method(&self, name: SymbolPtr, descriptor: SymbolPtr) -> Option<ResolvedMethod> {
    //     return Self::resolve_method_by_str(
    //         Ptr::from_ref(self),
    //         name.as_str(),
    //         descriptor.as_str(),
    //     );
    // }

    // pub fn get_method_by_str(&self, name: &str, descriptor: &str) -> Option<ResolvedMethod> {
    //     return Self::resolve_method_by_str(Ptr::from_ref(self), name, descriptor);
    // }

    pub fn is_primitive(&self) -> bool {
        return self.is_primitive;
    }

    pub fn is_array(&self) -> bool {
        return self.is_array;
    }

    pub fn is_implement(&self, other: ClassDataPtr) -> bool {
        let mut curr = ClassDataPtr::from_raw(self);
        loop {
            let vtab = curr.vtab();
            let ifaces = vtab.ifaces();
            let ifaces_len = vtab.ifaces_len();
            for idx in 0..ifaces_len {
                let iface = *ifaces.offset(idx as isize);
                if iface.class_data() == other {
                    return true;
                }
            }
            let super_class = curr.super_class();
            if super_class.is_not_null() {
                curr = super_class.class_data();
                continue;
            }
            break;
        }
        return false;
    }

    fn initialize(
        &mut self,
        jclass: JClassPtr,
        thread: ThreadPtr,
    ) -> Result<(), InitializationError> {
        let methods = self.methods;

        let vm = thread.vm();
        let vmstr_cls = vm.shared_objs().vm_str_cls;
        let jclass_cls = vm.preloaded_classes().jclass_cls();
        for idx in 0..methods.length() {
            let mut method: MethodPtr = methods.get(idx).cast();

            let params = method.params();
            for idx in 0..params.length() {
                let param = params.get(idx);
                if param.jclass() == vmstr_cls {
                    let param = vm
                        .bootstrap_class_loader
                        .load_class_with_symbol(param.cast())
                        .map_err(|_e| InitializationError::LinkingFailed)?;
                    params.set(idx, param.cast());
                } else {
                    debug_assert_eq!(param.jclass(), jclass_cls);
                }
            }

            if method.is_native() {
                let native_fn_name =
                    Self::get_native_fn_name(jclass.name().as_str(), method.name().as_str());
                if let Some(native_fn) = thread.vm().get_builtin_native_fn(&native_fn_name) {
                    method.set_native_fn(native_fn);
                }
            }
        }
        if self.is_array {
            self._vtab = vm.preloaded_classes().jobject_cls().class_data().vtab();
        } else {
            self.vtab()
                .initialize(self.super_class, self.methods, self.interfaces, vm);
        }
        return Ok(());
    }

    pub fn get_native_fn_name(class_name: &str, method_name: &str) -> String {
        let prefix = "Java_";
        let mut result =
            String::with_capacity(prefix.len() + class_name.len() + 1 + method_name.len());
        result.push_str(prefix);
        let mut last_end = 0;
        for (start, part) in class_name.match_indices('/') {
            result.push_str(unsafe { class_name.get_unchecked(last_end..start) });
            result.push('_');
            last_end = start + part.len();
        }
        result.push_str(unsafe { class_name.get_unchecked(last_end..class_name.len()) });
        result.push('_');
        result.push_str(method_name);
        result
    }

    pub fn debug(&self) {
        log::trace!(
            "vtab addr 0x{:x}, addr addr {:x?}, jobj vtab addr 0x{:x}",
            self.vtab().as_isize(),
            &self._vtab,
            Thread::current()
                .vm()
                .preloaded_classes()
                .jobject_cls()
                .class_data()
                .vtab()
                .as_isize()
        );
    }

    // fn new_array_class(class_name: SymbolPtr, element_size: usize, thread: ThreadPtr) -> ClassPtr {
    //     // let class_name = vm::instance().symbol_table.get_or_insert(name);
    //     // debug_assert_eq!(class_name.as_str(), name);
    //     let access_flags = ClassAccessFlags::AccPublic as u16 | ClassAccessFlags::AccFinal as u16;
    //     let mut class = Self::new_unparsed(thread);
    //     Class::parsed(
    //         class,
    //         ConstantPoolPtr::null(),
    //         access_flags,
    //         class_name,
    //         ClassPtr::null(),
    //         JArrayPtr::null(),
    //         thread.vm().shared_objs().empty_obj_arr.cast(),
    //         MethodArrayPtr::null(),
    //         ObjectPtr::null(),
    //         thread.vm().preloaded_classes().jclass_cls(),
    //     );
    //     class.ins_or_ele_size = element_size as u16;
    //     class.is_array = true;
    //     return class;
    // }

    // fn resolve_method_from_interfaces(
    //     interfaces: JArrayPtr,
    //     name: &str,
    //     descriptor: &str,
    // ) -> Result<ResolvedMethod, MethodResolutionError> {
    //     for i in 0..interfaces.length() {
    //         let interface: JClassPtr = interfaces.get_wrapped(i).cast();
    //         let interface_methods = interface.class().methods;
    //         for index in 0..interface_methods.length {
    //             let method = interface_methods.get_method(index as u16);
    //             if method.name.as_str() == name && method.descriptor.as_str() == descriptor {
    //                 return Ok(ResolvedMethod {
    //                     decl_class: interface.class(),
    //                     method,
    //                 });
    //             }
    //         }
    //         let super_interfaces = interface.class_data().interfaces();
    //         if super_interfaces.is_not_null() {
    //             return Self::resolve_method_from_interfaces(
    //                 interface.class_data().interfaces(),
    //                 name,
    //                 descriptor,
    //             );
    //         }
    //     }
    //     return Err(MethodResolutionError::NoSuchMethod);
    // }

    // fn setup_fields_layout(&mut self, heap: &Heap) {
    //     let fields = self.fields;
    //     if fields.is_null() {
    //         return;
    //     }
    //     // let mut padding: u16 = 0;
    //     // let mut offset: u16 = 0;
    //     // let mut aligned_offset: u16 = 0;
    //     let mut static_layout = FieldLayout::default();
    //     let mut non_static_layout = FieldLayout::default();

    //     if self.super_class.is_not_null() {
    //         non_static_layout.offset += self.super_class.ins_or_ele_size() as u16;
    //     }

    //     for field_it in 0..fields.length() {
    //         // todo
    //         let mut field: Ptr<Field> = fields.get_field(field_it);
    //         log::trace!(
    //             "{} is_static: {:b} field {} size {}",
    //             self.name.as_str(),
    //             field.is_static() as u8,
    //             field.name.as_str(),
    //             field.instance_size(),
    //         );
    //         if field.is_static() {
    //             Self::set_field_offset(&mut field, &mut static_layout);
    //         } else {
    //             Self::set_field_offset(&mut field, &mut non_static_layout);
    //         }
    //         // if prev_padding != 0 && field.class.is_primitive() {
    //         //     if global::classes::is_boolean_class(field.class) {
    //         //         prev_padding -= 1;

    //         //     }
    //         // }
    //         // self.instance_size.low =
    //         //     align((class.instance_size.low + field.instance_size()) as usize) as u32;
    //     }
    //     self.ins_or_ele_size = non_static_layout.aligned_offset as u16;
    //     if static_layout.offset != 0 {
    //         if !crate::memory::is_align_of(static_layout.offset as usize, POINTER_SIZE) {
    //             log::trace!("is_align_of {}", static_layout.offset);
    //             panic!("invalid size");
    //         }
    //         self.static_data =
    //             ObjectPtr::from_addr(heap.alloc_obj_permanent(static_layout.offset as usize));
    //     }
    // }

    // fn set_field_offset(field: &mut Field, layout: &mut FieldLayout) {
    //     let field_size = if field.field_class_is_primitive() {
    //         field.instance_size() as u16
    //     } else {
    //         std::mem::size_of::<Address>() as u16
    //     };
    //     if layout.padding >= field_size {
    //         layout.padding -= field_size;
    //         field.offset = layout.offset;
    //         layout.offset += field_size;
    //     } else if field_size < Self::FIELD_ALIGNMENT {
    //         layout.padding = Self::FIELD_ALIGNMENT - field_size;
    //         field.offset = layout.aligned_offset;
    //         layout.aligned_offset += Self::FIELD_ALIGNMENT;
    //         layout.offset += field_size;
    //     } else {
    //         layout.padding = 0;
    //         field.offset = layout.aligned_offset;
    //         layout.aligned_offset += field_size;
    //         layout.offset = layout.aligned_offset;
    //     }
    // }

    fn vtab(&self) -> VTablePtr {
        debug_assert!(self._vtab.is_not_null());
        return self._vtab;
    }

    fn vtab_slow(class_data: ClassDataPtr) -> VTablePtr {
        return VTablePtr::from_addr(
            class_data
                .as_address()
                .offset(std::mem::size_of::<ClassData>() as isize),
        );
    }

    const fn size(vtab_len: u32, ifaces_len: u32, ifaces_m_indexes_len: u32) -> u32 {
        return align(
            std::mem::size_of::<ClassData>()
                + VTable::size(vtab_len, ifaces_len, ifaces_m_indexes_len),
        ) as u32;
    }
}

#[derive(PartialEq, Debug)]
#[repr(u8)]
enum ClassInitState {
    Created,
    Linked,
    Initializing,
    Initialized,
}

impl ClassInitState {
    fn as_u8(&self) -> u8 {
        return unsafe { std::mem::transmute_copy(self) };
    }
}

// JClass layout
//  -------------------------------------
// |                 Header              |
//  -------------------------------------
// |           JClass(Rust) Fields       |
//  -------------------------------------
// |  java/lang/Class Non-Static Fields  |
//  -------------------------------------
// |               ClassData             |
//  -------------------------------------
// |        Real Class Static Fields     |
//  -------------------------------------
define_oop!(
    struct JClass {
        _init_state: ClassInitState,
        class_data: ClassDataPtr,
    }
);

impl JClass {
    pub fn new_permanent(
        cp: ConstantPoolPtr,
        access_flags: u16,
        name: SymbolPtr,
        super_class: JClassPtr,
        interfaces: JArrayPtr,
        fields: JArrayPtr,
        methods: JArrayPtr,
        static_fields_size: u16,
        vtab_info: &VTableInfo,
        inst_size: u16,
        metadata_offset: u16,
        jclass_loader: ObjectPtr,
        init_method: MethodPtr,
        component_type: JClassPtr,
        jclass_jclass: JClassPtr,
        thread: ThreadPtr,
    ) -> JClassPtr {
        let vtab_len = vtab_info.methods.len();
        let ifaces_len = vtab_info.ifaces.len();
        let ifaces_m_indexes_len = vtab_info.ifaces_m_indexes.len();
        let jclass = thread.heap().alloc_cls_permanent(
            cp,
            access_flags,
            name,
            super_class,
            interfaces,
            fields,
            methods,
            thread.vm().shared_objs().java_lang_class_inst_size(),
            static_fields_size,
            vtab_len as u32,
            ifaces_len as u32,
            ifaces_m_indexes_len as u32,
            inst_size,
            metadata_offset,
            jclass_loader,
            init_method,
            component_type,
            thread,
        );

        for idx in 0..methods.length() {
            let method: MethodPtr = methods.get(idx).cast();
            method.as_mut_ref().set_decl_cls(jclass);
        }
        let vtab = jclass.class_data().vtab();

        unsafe {
            std::ptr::copy(
                vtab_info.methods.as_ptr(),
                vtab.methods().as_mut_raw_ptr(),
                vtab_len,
            );
            std::ptr::copy(
                vtab_info.ifaces.as_ptr(),
                vtab.ifaces().as_mut_raw_ptr(),
                ifaces_len,
            );
            std::ptr::copy(
                vtab_info.ifaces_m_indexes.as_ptr(),
                vtab.imethod_indexes().as_mut_raw_ptr(),
                ifaces_m_indexes_len,
            );
        }
        return jclass;
    }

    pub fn new_system_class(
        name: SymbolPtr,
        instance_size: usize,
        is_primitive: bool,
        is_array: bool,
        component_type: JClassPtr,
        thread: ThreadPtr,
    ) -> JClassPtr {
        let access_flags = ClassAccessFlags::AccPublic as u16 | ClassAccessFlags::AccFinal as u16;
        let jclass = thread.heap().alloc_cls_permanent(
            ConstantPoolPtr::null(),
            access_flags,
            name,
            JClassPtr::null(),
            JArrayPtr::null(),
            thread.vm().shared_objs().empty_sys_arr,
            thread.vm().shared_objs().empty_sys_arr,
            thread.vm().shared_objs().java_lang_class_inst_size(),
            0,
            0,
            0,
            0,
            instance_size as u16,
            0,
            ObjectPtr::null(),
            MethodPtr::null(),
            component_type,
            thread,
        );
        jclass.class_data().is_primitive = is_primitive;
        jclass.class_data().is_array = is_array;

        // let class_name = vm::instance().symbol_table.get_or_insert(name);
        // debug_assert_eq!(class_name.as_str(), name);
        // let access_flags = ClassAccessFlags::AccPublic as u16 | ClassAccessFlags::AccFinal as u16;
        // let mut class = ClassPtr::from_addr(thread.heap().alloc_obj_permanent(size_of::<Class>()));
        // class.access_flags = access_flags;
        // class.name = class_name;
        // class.ins_or_ele_size = instance_size as u16;
        // class.is_primitive = true;
        log::trace!(
            "new_system_class {}, cls addr {:x}, name addr {:x}",
            jclass.class_data().name().as_str(),
            jclass.as_usize(),
            jclass.class_data().name().as_usize()
        );
        return jclass;
    }

    pub fn new_array_class(
        name: SymbolPtr,
        component_type: JClassPtr,
        thread: ThreadPtr,
    ) -> JClassPtr {
        let vm = thread.vm();
        let jobj_cls = vm.preloaded_classes().jobject_cls();
        debug_assert!(
            jobj_cls.is_not_null()
                && jobj_cls.class_data().vtab().is_not_null()
                && jobj_cls.class_data().vtab().vtab_len > 0
        );
        let access_flags = ClassAccessFlags::AccPublic as u16 | ClassAccessFlags::AccFinal as u16;
        let jclass = thread.heap().alloc_cls_permanent(
            ConstantPoolPtr::null(),
            access_flags,
            name,
            jobj_cls,
            JArrayPtr::null(),
            JArrayPtr::null(),
            JArrayPtr::null(),
            vm.shared_objs().java_lang_class_inst_size(),
            0,
            0,
            0,
            0,
            0,
            0,
            component_type.class_data().jclass_loader,
            MethodPtr::null(),
            component_type,
            thread,
        );
        jclass.as_mut_ref()._init_state = ClassInitState::Linked;
        jclass.class_data().is_array = true;
        jclass.class_data()._vtab = jobj_cls.class_data().vtab();
        return jclass;
    }

    pub fn new_vm_internal_class(
        name: SymbolPtr,
        is_array: bool,
        component_type: JClassPtr,
        thread: ThreadPtr,
    ) -> JClassPtr {
        let access_flags = ClassAccessFlags::AccPublic as u16 | ClassAccessFlags::AccFinal as u16;
        let jclass = thread.heap().alloc_cls_permanent(
            ConstantPoolPtr::null(),
            access_flags,
            name,
            JClassPtr::null(),
            JArrayPtr::null(),
            JArrayPtr::null(),
            JArrayPtr::null(),
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            ObjectPtr::null(),
            MethodPtr::null(),
            component_type,
            thread,
        );
        jclass.class_data().is_array = is_array;

        // let class_name = vm::instance().symbol_table.get_or_insert(name);
        // debug_assert_eq!(class_name.as_str(), name);
        // let access_flags = ClassAccessFlags::AccPublic as u16 | ClassAccessFlags::AccFinal as u16;
        // let mut class = ClassPtr::from_addr(thread.heap().alloc_obj_permanent(size_of::<Class>()));
        // class.access_flags = access_flags;
        // class.name = class_name;
        // class.ins_or_ele_size = instance_size as u16;
        // class.is_primitive = true;
        log::trace!(
            "new_vm_internal_class {}, cls addr {:x}, name addr {:x}, name hash {}",
            jclass.class_data().name().as_str(),
            jclass.as_usize(),
            jclass.class_data().name().as_usize(),
            jclass.class_data().name().hash_code()
        );
        return jclass;
    }

    pub fn initialize(&self, thread: ThreadPtr) -> Result<(), InitializationError> {
        if self._init_state == ClassInitState::Initialized {
            return Ok(());
        }
        if !self.is_linked() {
            self.link(thread)?;
        }
        // TODO: the initialization of a class or interface must be synchronized.
        if self._init_state == ClassInitState::Initializing {
            return Ok(());
        }
        let mut self_ptr = JClassPtr::from_ref(self);
        self_ptr._init_state = ClassInitState::Initializing;
        let init_method = self.class_data().init_method;
        if init_method.is_not_null() {
            thread
                .vm()
                .call_static_void(JClassPtr::from_ref(self), init_method, &[]);
        }
        self_ptr._init_state = ClassInitState::Initialized;
        return Ok(());
    }

    pub fn is_void(cls: JClassPtr, vm: VMPtr) -> bool {
        return vm.preloaded_classes().is_void_cls(cls);
    }

    pub fn is_long(cls: JClassPtr, vm: VMPtr) -> bool {
        return vm.preloaded_classes().is_long_cls(cls);
    }

    pub fn is_double(cls: JClassPtr, vm: VMPtr) -> bool {
        return vm.preloaded_classes().is_double_cls(cls);
    }

    pub fn is_long_arr(cls: JClassPtr, vm: VMPtr) -> bool {
        return vm.preloaded_classes().is_long_arr_cls(cls);
    }
    pub fn is_double_arr(cls: JClassPtr, vm: VMPtr) -> bool {
        return vm.preloaded_classes().is_double_arr_cls(cls);
    }
    pub fn is_int_arr(cls: JClassPtr, vm: VMPtr) -> bool {
        return vm.preloaded_classes().is_int_arr_cls(cls);
    }
    pub fn is_float_arr(cls: JClassPtr, vm: VMPtr) -> bool {
        return vm.preloaded_classes().is_float_arr_cls(cls);
    }
    pub fn is_short_arr(cls: JClassPtr, vm: VMPtr) -> bool {
        return vm.preloaded_classes().is_short_arr_cls(cls);
    }
    pub fn is_char_arr(cls: JClassPtr, vm: VMPtr) -> bool {
        return vm.preloaded_classes().is_char_arr_cls(cls);
    }
    pub fn is_byte_arr(cls: JClassPtr, vm: VMPtr) -> bool {
        return vm.preloaded_classes().is_byte_arr_cls(cls);
    }
    pub fn is_boolean_arr(cls: JClassPtr, vm: VMPtr) -> bool {
        return vm.preloaded_classes().is_bool_arr_cls(cls);
    }

    pub fn is_primitive(cls: JClassPtr) -> bool {
        return cls.class_data().is_primitive();
    }

    pub fn get_primitive_class(name: SymbolPtr, vm: VMPtr) -> JClassPtr {
        return vm.preloaded_classes().get_primitive_class(name);
    }

    pub fn name(&self) -> SymbolPtr {
        return self.class_data().name();
    }

    pub fn class_loader(&self) -> ObjectPtr {
        return self.class_data().jclass_loader;
    }

    pub fn is_initialized(&self) -> bool {
        return self._init_state != ClassInitState::Created;
    }

    pub fn class_data(&self) -> ClassDataPtr {
        return self.class_data;
    }

    pub fn set_class_data(cls: JClassPtr, java_lang_class_inst_size: u16) {
        cls.as_mut_ref().class_data = ClassDataPtr::from_addr(
            cls.as_address()
                .offset(Self::class_data_offset(java_lang_class_inst_size) as isize),
        );
    }

    pub fn is_implement(&self, other: JClassPtr) -> bool {
        if other.is_null() {
            return false;
        }
        return self.class_data().is_implement(other.class_data());
    }

    pub fn is_assignable_from(&self, target: JClassPtr, vm: VMPtr) -> bool {
        let self_cls = JClassPtr::from_ref(self);
        if self_cls == target {
            return true;
        }
        let target_cls_data = target.class_data();
        if target_cls_data.is_interface() {
            if self_cls == vm.preloaded_classes().jobject_cls() {
                return true;
            }
            return target.is_implement(self_cls);
        } else if target_cls_data.is_array() {
            return self_cls == vm.preloaded_classes().jobject_cls();
        }
        if self_cls.class_data().is_interface() {
            return target.is_implement(self_cls);
        }
        let mut target_super_class = target.class_data().super_class();
        loop {
            if target_super_class.is_null() {
                return false;
            }
            if target_super_class == self_cls {
                return true;
            }
            target_super_class = target_super_class.class_data().super_class();
        }
    }

    pub fn is_unlinked_symbol(&self, thread: ThreadPtr) -> bool {
        return thread.vm().shared_objs().vm_str_cls == JClassPtr::from_ref(self);
    }

    pub fn get_static_value(&self, offset: i32, val_bytes: i32) -> i64 {
        let dst: ObjectPtr = ObjectPtr::from_ref(self).cast();
        return dst.read_value(offset, val_bytes);
    }

    pub fn set_static_value<T>(&self, offset: i32, val: T) {
        let dst: ObjectPtr = ObjectPtr::from_addr(Address::from_ref(self).offset(offset as isize));
        unsafe {
            std::ptr::write(dst.cast::<T>().as_mut_raw_ptr(), val);
        }
    }

    pub fn get_field(&self, field_ref: &ConstMemberRef) -> (FieldPtr, JClassPtr) {
        return self.get_field_with_name(field_ref.member_name);
    }

    pub fn get_field_with_name(&self, field_name: SymbolPtr) -> (FieldPtr, JClassPtr) {
        let mut lookup_cls = JClassPtr::from_ref(self);
        loop {
            let class_data = lookup_cls.class_data();
            let fields = class_data.fields();
            for i in 0..fields.length() {
                let field: FieldPtr = fields.get(i).cast();
                if field.name() == field_name {
                    return (field, lookup_cls);
                }
            }
            if class_data.super_class().is_not_null() {
                lookup_cls = class_data.super_class();
            } else {
                break;
            }
        }

        log::trace!("get_field {} not found", field_name.as_str());
        return (FieldPtr::null(), JClassPtr::null());
    }

    pub fn get_field_val(
        &self,
        obj: ObjectPtr,
        field_ref: &ConstMemberRef,
        thread: ThreadPtr,
    ) -> Result<i64, ClassLoadErr> {
        let (field, _) = self.get_field(field_ref);
        return Ok(field.get_value(obj, thread)?);
    }

    pub fn resolve_interface_method(
        class: JClassPtr,
        iface: JClassPtr,
        name: SymbolPtr,
        descriptor: SymbolPtr,
    ) -> Result<ResolvedMethod, MethodResolutionError> {
        let mut class_data = class.class_data();
        let class_v_methods = class_data.vtab().methods();
        loop {
            let vtab = class_data.vtab();
            let ifaces = vtab.ifaces();
            let imethod_indexes = vtab.imethod_indexes();
            let imethod_indexes_offset = 0isize;
            for if_idx in 0..vtab.ifaces_len() {
                let impl_iface = *ifaces.offset(if_idx as isize);
                log::trace!(
                    "resolve_interface_method class {}, iface {}, name: {}, descriptor {}",
                    class_data.name().as_str(),
                    impl_iface.name().as_str(),
                    name.as_str(),
                    descriptor.as_str(),
                );
                if impl_iface == iface {
                    let mut imethod_idx = -1;
                    impl_iface.resolve_local_method(name, descriptor, &mut imethod_idx);
                    if imethod_idx >= 0 {
                        let v_method_idx =
                            *imethod_indexes.offset(imethod_indexes_offset + imethod_idx as isize);
                        log::trace!(
                                "resolve_interface_method class {} success, v_method_idx {}, v_method addr 0x{:x}",
                                class_data.name().as_str(),
                                v_method_idx,
                                (*class_v_methods.offset(v_method_idx as isize)).as_isize()
                            );
                        return Ok(ResolvedMethod {
                            decl_class: JClassPtr::null(),
                            method: *class_v_methods.offset(v_method_idx as isize),
                            method_idx: v_method_idx,
                        });
                    } else {
                        log::trace!("resolve specific method failed {}", imethod_idx);
                        JClass::debug(impl_iface);
                        return Err(MethodResolutionError::NoSuchMethod);
                    }
                }
            }
            if class_data.super_class().is_not_null() {
                class_data = class_data.super_class().class_data();
            } else {
                return Err(MethodResolutionError::IncompatibleClassChange);
            }
        }
    }

    pub fn resolve_virtual_with_index(
        objref: ObjectPtr,
        method: MethodPtr,
        method_idx: u32,
    ) -> Result<ResolvedMethod, MethodResolutionError> {
        debug_assert!(objref.is_not_null());
        let vtab = objref.jclass().class_data().vtab();
        let v_methods_len = vtab.vtab_len;
        if method_idx as u32 >= v_methods_len {
            log::trace!(
                "resolve_virtual_with_index failed, objref jclass {}, method_idx {} >= v_methods_len {}, vtab addr 0x{:x}, jobject vtab addr 0x{:x}",
                objref.jclass().name().as_str(),
                method_idx,
                v_methods_len,
                vtab.as_isize(),
                Thread::current().vm().preloaded_classes().jobject_cls().class_data().vtab().as_isize()
            );
            return Err(MethodResolutionError::AbstractMethod);
        }
        let v_methods = vtab.methods();
        let resolved_method = *v_methods.offset(method_idx as isize);
        if resolved_method.name() != method.name()
            || resolved_method.descriptor() != method.descriptor()
        {
            log::trace!(
                "resolved_method.name() {} != method.name() {} || resolved_method.descriptor() {} != method.descriptor() {}",
                resolved_method.name().as_str(),
                method.name().as_str(),
                resolved_method.descriptor().as_str(),
                method.descriptor().as_str()
            );
            return Err(MethodResolutionError::AbstractMethod);
        }
        if resolved_method.is_abstract() {
            log::trace!(
                "resolved_method.is_abstract() m {}#{}, v_m {}#{}",
                method.decl_cls().name().as_str(),
                method.name().as_str(),
                resolved_method.decl_cls().name().as_str(),
                resolved_method.descriptor().as_str()
            );
            return Err(MethodResolutionError::AbstractMethod);
        }
        return Ok(ResolvedMethod {
            decl_class: objref.jclass(),
            method: resolved_method,
            method_idx: method_idx,
        });
    }

    pub fn resolve_local_method_unchecked(
        &self,
        name: SymbolPtr,
        descriptor: SymbolPtr,
    ) -> MethodPtr {
        let methods = self.class_data().methods;
        for idx in 0..methods.length() {
            let method: MethodPtr = methods.get(idx).cast();
            if method.name() == name && method.descriptor() == descriptor {
                return method;
            }
        }
        return MethodPtr::null();
    }

    pub fn resolve_local_method(
        &self,
        name: SymbolPtr,
        descriptor: SymbolPtr,
        method_idx: &mut MethodIndex,
    ) -> MethodPtr {
        let methods = self.class_data().methods;
        for idx in 0..methods.length() {
            let method: MethodPtr = methods.get(idx).cast();
            if method.name() == name && method.descriptor() == descriptor {
                *method_idx = idx;
                return method;
            }
        }
        return MethodPtr::null();
    }

    pub fn resolve_class_method(
        &self,
        name: SymbolPtr,
        descriptor: SymbolPtr,
        vm: &VM,
    ) -> Result<ResolvedMethod, MethodResolutionError> {
        if name == vm.shared_objs().symbols().ctor_init {
            return self.resolve_self_method(name, descriptor);
        }
        let vtab = self.class_data().vtab();
        let v_methods = vtab.methods();
        let vtab_len = vtab.vtab_len;
        for idx in 0..vtab_len {
            let v_method = *v_methods.offset(idx as isize);
            if v_method.name() == name && v_method.descriptor() == descriptor {
                return Ok(ResolvedMethod {
                    decl_class: JClassPtr::null(),
                    method: v_method,
                    method_idx: idx,
                });
            }
        }
        log::trace!(
            "resolve_class_method failed {}, name: {}, descriptor {}",
            self.name().as_str(),
            name.as_str(),
            descriptor.as_str()
        );
        return Err(MethodResolutionError::NoSuchMethod);
    }

    pub fn resolve_self_method(
        &self,
        name: SymbolPtr,
        descriptor: SymbolPtr,
    ) -> Result<ResolvedMethod, MethodResolutionError> {
        let methods = self.class_data().methods();
        for idx in 0..methods.length() {
            let method: MethodPtr = methods.get(idx).cast();
            if method.name() == name && method.descriptor() == descriptor {
                return Ok(ResolvedMethod {
                    decl_class: JClassPtr::null(),
                    method: method,
                    method_idx: idx as _,
                });
            }
        }
        log::trace!(
            "resolve_special_method failed {}, name: {}, descriptor {}",
            self.name().as_str(),
            name.as_str(),
            descriptor.as_str()
        );
        return Err(MethodResolutionError::NoSuchMethod);
    }

    pub fn get_method_with_index(&self, method_idx: JInt) -> MethodPtr {
        let methods = self.class_data().methods();
        if method_idx < methods.length() {
            return methods.get(method_idx).cast();
        }
        return MethodPtr::null();
    }

    pub fn ref_size(jclass: JClassPtr) -> usize {
        if jclass.is_null() {
            // unresolved
            return size_of::<Address>();
        }
        let class_data = jclass.class_data();
        if class_data.is_primitive() {
            if usize::from(class_data.inst_or_ele_size) > 8 {
                log::trace!("class  {} inst_or_ele_size > 8 ", class_data.name.as_str());
            }
            return usize::from(class_data.inst_or_ele_size);
        }
        return size_of::<Address>();
    }

    pub fn debug(jclass: JClassPtr) {
        let methods = jclass.class_data().methods;
        let class_name = jclass.name();
        for index in 0..methods.length() {
            let method: MethodPtr = methods.get(index).cast();
            log::trace!(
                "debug class {} method {}, method addr 0x{:x}, descriptor {}",
                class_name.as_str(),
                method.name().as_str(),
                method.as_isize(),
                method.descriptor().as_str()
            );
        }
    }

    // fn resolve_method_by_str(
    //     class: JClassPtr,
    //     name: &str,
    //     descriptor: &str,
    // ) -> Result<ResolvedMethod, MethodResolutionError> {
    //     let mut current_class = class;
    //     loop {
    //         let class_data = current_class.class_data();
    //         let methods = class_data.methods;
    //         for index in 0..methods.length() {
    //             let method: MethodPtr = methods.get(index).cast();
    //             if method.name().as_str() == name && method.descriptor().as_str() == descriptor {
    //                 return Ok(ResolvedMethod {
    //                     decl_class: current_class,
    //                     method,
    //                 });
    //             }
    //         }
    //         if class_data.super_class().is_not_null() {
    //             current_class = class_data.super_class();
    //             continue;
    //         }
    //         break;
    //     }
    //     if class.class_data().interfaces().is_not_null() {
    //         return Self::resolve_method_from_interfaces(
    //             class.class_data().interfaces(),
    //             name,
    //             descriptor,
    //         );
    //     }
    //     return Err(MethodResolutionError::NoSuchMethod);
    // }

    // fn resolve_method_from_interfaces(
    //     interfaces: JArrayPtr,
    //     name: &str,
    //     descriptor: &str,
    // ) -> Result<ResolvedMethod, MethodResolutionError> {
    //     for i in 0..interfaces.length() {
    //         let interface: JClassPtr = interfaces.get(i).cast();
    //         let interface_methods = interface.class_data().methods;
    //         for index in 0..interface_methods.length() {
    //             let method: MethodPtr = interface_methods.get(index).cast();
    //             if method.name().as_str() == name && method.descriptor().as_str() == descriptor {
    //                 return Ok(ResolvedMethod {
    //                     decl_class: interface,
    //                     method,
    //                     method_idx: index as _,
    //                 });
    //             }
    //         }
    //         let super_interfaces = interface.class_data().interfaces();
    //         if super_interfaces.is_not_null() {
    //             return Self::resolve_method_from_interfaces(
    //                 interface.class_data().interfaces(),
    //                 name,
    //                 descriptor,
    //             );
    //         }
    //     }
    //     return Err(MethodResolutionError::NoSuchMethod);
    // }

    fn link(&self, thread: ThreadPtr) -> Result<(), InitializationError> {
        debug_assert!(self._init_state == ClassInitState::Created);
        let mut self_ptr = JClassPtr::from_ref(self);
        // TODO: the initialization of a class or interface must be synchronized.
        let class_data = self.class_data();
        log::trace!("link {}", class_data.name.as_str());
        if class_data.is_interface() {
            self_ptr._init_state = ClassInitState::Linked;
            return Ok(());
        }
        let super_class = class_data.super_class();
        if super_class.is_not_null() && !super_class.is_linked() {
            log::trace!(
                "link super_class {}",
                super_class.class_data().name.as_str()
            );
            super_class.link(thread)?;
        }
        class_data.as_mut_ref().initialize(self_ptr, thread)?;
        self.adjust_fields(thread)?;
        self_ptr._init_state = ClassInitState::Linked;
        return Ok(());
    }

    fn is_linked(&self) -> bool {
        return self._init_state.as_u8() >= ClassInitState::Linked.as_u8();
    }

    fn adjust_fields(&self, thread: ThreadPtr) -> Result<(), InitializationError> {
        let self_ptr = JClassPtr::from_ref(self);
        let java_lang_class = thread.vm().preloaded_classes().jclass_cls();
        let non_static_fields_offset = if java_lang_class == self_ptr {
            Self::class_non_static_offset() as u16
        } else if self.class_data().super_class().is_not_null() {
            let super_inst_size = self
                .class_data()
                .super_class()
                .class_data()
                .inst_or_ele_size;
            self_ptr.class_data().inst_or_ele_size += super_inst_size;
            std::mem::size_of::<Object>() as u16 + super_inst_size
        } else {
            std::mem::size_of::<Object>() as u16
        };
        let static_fields_offset = {
            let vtab = self.class_data().vtab();
            Self::class_static_fields_offset(
                thread.vm().shared_objs().java_lang_class_inst_size(),
                vtab.vtab_len,
                vtab.ifaces_len,
                vtab.ifaces_methods_len,
            ) as u16
        };
        log::trace!(
            "adjust_fields_offset {}, static_fields_offset {}",
            self.name().as_str(),
            static_fields_offset
        );
        let fields = self.class_data().fields();
        let vm = thread.vm();
        for field_idx in 0..fields.length() {
            let field: FieldPtr = fields.get(field_idx).cast();
            if field.field_class_unchecked().is_null() {
                let field_cls = vm
                    .bootstrap_class_loader
                    .load_class_with_symbol(field.descriptor())
                    .map_err(|_e| InitializationError::LinkingFailed)?;
                field.as_mut_ref().set_field_class(field_cls);
            }
            if field.is_static() {
                field
                    .as_mut_ref()
                    .set_layout_offset(field.layout_offset() + static_fields_offset);
                log::trace!(
                    "adjust_fields_offset {}, field {}, offset {}",
                    self.name().as_str(),
                    field.name().as_str(),
                    field.layout_offset()
                );
            } else {
                field
                    .as_mut_ref()
                    .set_layout_offset(field.layout_offset() + non_static_fields_offset);
            }
        }
        return Ok(());
    }

    const fn class_non_static_offset() -> u32 {
        return std::mem::size_of::<JClass>() as u32;
    }

    const fn class_data_offset(java_lang_class_inst_size: u16) -> u32 {
        return Self::class_non_static_offset() + java_lang_class_inst_size as u32;
    }

    const fn class_static_fields_offset(
        java_lang_class_inst_size: u16,
        vtab_len: u32,
        ifaces_len: u32,
        ifaces_m_indexes_len: u32,
    ) -> u32 {
        return Self::class_data_offset(java_lang_class_inst_size)
            + ClassData::size(vtab_len, ifaces_len, ifaces_m_indexes_len);
    }

    pub fn size(
        java_lang_class_inst_size: u16,
        static_fields_size: u16,
        vtab_len: u32,
        ifaces_len: u32,
        ifaces_m_indexes_len: u32,
    ) -> u32 {
        return Self::class_static_fields_offset(
            java_lang_class_inst_size,
            vtab_len,
            ifaces_len,
            ifaces_m_indexes_len,
        ) + u32::from(static_fields_size);
    }
}

impl VMObject for JClass {
    fn hash(obj: ObjectPtr) -> JInt {
        return obj.cast::<JClass>().name().hash_code();
    }

    fn equals(obj: ObjectPtr, other: ObjectPtr) -> bool {
        return obj.cast::<JClass>().name() == other.cast::<JClass>().name();
    }
}

impl<'a> GetEntryWithKey<Utf8String<'a>> for JClass {
    fn hash_key(ref_str: Utf8String) -> i32 {
        return Symbol::hash_utf8(ref_str.value);
    }

    fn entry_equals_key(value: Address, ref_str: Utf8String) -> bool {
        let class = JClassPtr::from_addr(value);
        return class.name().equals_utf8(ref_str);
    }
}

#[derive(Default)]
pub struct FieldLayout {
    padding: u16,
    offset: u16,
    aligned_offset: u16,
}

impl FieldLayout {
    const FIELD_ALIGNMENT: u16 = 8;

    pub fn obtain_field_offset(&mut self, field_val_size: u16) -> u16 {
        // TODO
        let offset: u16;
        if self.padding >= field_val_size {
            self.padding -= field_val_size;
            offset = self.offset;
            self.offset += field_val_size;
        } else if field_val_size < Self::FIELD_ALIGNMENT {
            self.padding = Self::FIELD_ALIGNMENT - field_val_size;
            offset = self.aligned_offset;
            self.aligned_offset += Self::FIELD_ALIGNMENT;
            self.offset += field_val_size;
        } else {
            self.padding = 0;
            offset = self.aligned_offset;
            self.aligned_offset += field_val_size;
            self.offset = self.aligned_offset;
        }
        return offset;
    }

    pub fn get_aligned_size(&self) -> u16 {
        return self.aligned_offset;
    }
}

#[derive(Debug)]
pub enum InitializationError {
    ResolveError(MethodResolutionError),
    LinkingFailed,
}

#[derive(Debug)]
pub enum MethodResolutionError {
    IncompatibleClassChange,
    NoSuchMethod,
    AbstractMethod,
    IllegalAccess,
}
