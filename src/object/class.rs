use super::prelude::*;
use super::ptr::Ptr;
use super::{Header, ObjectTag};
use crate::global;
use crate::memory::Address;
use crate::vm;
use std::convert::From;
use std::mem::size_of;

pub type ClassPtr = Ptr<Class>;
pub type ConstantPoolPtr = Ptr<ConstantPool>;
pub type FieldPtr = Ptr<Field>;
pub type FieldArrayPtr = Ptr<FieldArray>;
pub type MethodPtr = Ptr<Method>;
pub type MethodArrayPtr = Ptr<MethodArray>;

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

pub struct Class {
    header: Header,
    access_flags: u16,
    cp: ConstantPoolPtr,
    name: SymbolPtr,
    super_class: ClassPtr,
    interfaces: JRefArrayPtr,
    fields: FieldArrayPtr,
    methods: MethodArrayPtr,
}

impl Class {
    pub fn new_permanent(
        cp: ConstantPoolPtr,
        access_flags: u16,
        name: SymbolPtr,
        super_class: ClassPtr,
        interfaces: JRefArrayPtr,
        fields: FieldArrayPtr,
        methods: MethodArrayPtr,
    ) -> ClassPtr {
        let mut class =
            ClassPtr::from_addr(vm::instance().heap.alloc_obj_permanent(size_of::<Class>()));
        class.access_flags = access_flags;
        class.cp = cp;
        class.name = name;
        class.super_class = super_class;
        class.interfaces = interfaces;
        class.fields = fields;
        class.methods = methods;
        class.cast::<Object>().initialize(global::classes::cclass());
        return class;
    }

    pub fn is_interface(&self) -> bool {
        self.access_flags & ClassAccessFlags::AccInterface as u16
            == ClassAccessFlags::AccInterface as u16
    }

    pub fn name(&self) -> SymbolPtr {
        self.name
    }

    pub fn bootstrap(&mut self, class: ClassPtr) {
        self.header.class = class;
    }
}

#[repr(u8)]
pub enum ConstantTag {
    Invalid = 0,
    Utf8 = 1,
    Integer = 3,
    Float = 4,
    Long = 5,
    Double = 6,
    Class = 7,
    String = 8,
    Fieldref = 9,
    Methodref = 10,
    InterfaceMethodref = 11,
    NameAndType = 12,
    MethodHandle = 15,
    MethodType = 16,
    InvokeDynamic = 18,

    // rsvm specific tags
    ClassName = 101,
}

impl From<u8> for ConstantTag {
    fn from(tag: u8) -> Self {
        unsafe { std::mem::transmute(tag) }
    }
}

pub struct ConstantPool {
    header: Header,
    tags: JByteArrayPtr,
    info: *mut u64,
}

impl ConstantPool {
    pub const CP_CLASS: ClassPtr =
        ClassPtr::from_addr(Address::from_usize(ObjectTag::ConstantPool as usize));

    pub fn new(length: u16) -> ConstantPoolPtr {
        let size = ConstantPool::object_size(length);
        let mut cp = ConstantPoolPtr::from_addr(vm::instance().heap.alloc_obj_permanent(size));
        cp.header.initialize(ConstantPool::CP_CLASS);
        // TODO write barrier
        cp.tags = JByteArray::new_permanent(length as JInt);
        return cp;
    }

    fn object_size(length: u16) -> usize {
        Header::size() + size_of::<JByteArrayPtr>() + length as usize * size_of::<u64>()
    }

    pub fn size(&self) -> u16 {
        self.tags.length() as u16
    }

    pub fn set_invalid(&self, index: u16) {
        self.tags.set(index as JInt, ConstantTag::Invalid as JByte);
    }

    pub fn set_utf8(&mut self, index: u16, value: SymbolPtr) {
        self.tags.set(index as JInt, ConstantTag::Utf8 as JByte);
        unsafe {
            std::ptr::write(self.info.offset(index as isize), value.as_usize() as u64);
        }
    }

    pub fn get_utf8(&self, index: u16) -> SymbolPtr {
        assert!(self.tags.get(index as JInt) == ConstantTag::Utf8 as JByte);
        unsafe {
            let addr = std::ptr::read(self.info.offset(index as isize)) as usize;
            return SymbolPtr::from_usize(addr);
        }
    }

    pub fn set_int32(&mut self, index: u16, value: JInt) {
        self.tags.set(index as JInt, ConstantTag::Integer as JByte);
        unsafe {
            std::ptr::write(self.info.offset(index as isize), value as u64);
        }
    }

    pub fn set_float(&mut self, index: u16, value: JFloat) {
        self.tags.set(index as JInt, ConstantTag::Float as JByte);
        unsafe {
            std::ptr::write(self.info.offset(index as isize), value as u64);
        }
    }

    pub fn set_long(&mut self, index: u16, value: JLong) {
        self.tags.set(index as JInt, ConstantTag::Long as JByte);
        unsafe {
            std::ptr::write(self.info.offset(index as isize), value as u64);
        }
    }

    pub fn set_double(&mut self, index: u16, value: JDouble) {
        self.tags.set(index as JInt, ConstantTag::Double as JByte);
        unsafe {
            std::ptr::write(self.info.offset(index as isize), value as u64);
        }
    }

    pub fn set_class_index(&mut self, index: u16, class_index: u16) {
        self.tags.set(index as JInt, ConstantTag::Class as JByte);
        unsafe {
            std::ptr::write(self.info.offset(index as isize), class_index as u64);
        }
    }

    pub fn get_class_name(&self, index: u16) -> SymbolPtr {
        let index_tag = self.tags.get(index as i32);
        assert!(index_tag == ConstantTag::Class as JByte);
        unsafe {
            let name_index = std::ptr::read(self.info.offset(index as isize));
            assert!((name_index as i32) < self.tags.length());
            let name_index_tag = self.tags.get(name_index as JInt);
            assert!(name_index_tag == ConstantTag::Utf8 as JByte);
            let addr = std::ptr::read(self.info.offset(name_index as isize)) as usize;
            return SymbolPtr::from_usize(addr);
        }
    }

    pub fn set_string(&mut self, index: u16, string_index: u16) {
        self.tags.set(index as JInt, ConstantTag::String as JByte);
        unsafe {
            std::ptr::write(self.info.offset(index as isize), string_index as u64);
        }
    }

    pub fn set_field_ref(&mut self, index: u16, class_index: u16, name_and_type_index: u16) {
        self.tags.set(index as JInt, ConstantTag::Fieldref as JByte);
        let encoded_value = ((class_index as u64) << 16) | (name_and_type_index as u64);
        unsafe {
            std::ptr::write(self.info.offset(index as isize), encoded_value);
        }
    }

    pub fn set_method_ref(&mut self, index: u16, class_index: u16, name_and_type_index: u16) {
        self.tags
            .set(index as JInt, ConstantTag::Methodref as JByte);
        let encoded_value = ((class_index as u64) << 16) | (name_and_type_index as u64);
        unsafe {
            std::ptr::write(self.info.offset(index as isize), encoded_value);
        }
    }

    pub fn set_interface_method_ref(
        &mut self,
        index: u16,
        class_index: u16,
        name_and_type_index: u16,
    ) {
        self.tags
            .set(index as JInt, ConstantTag::InterfaceMethodref as JByte);
        let encoded_value = ((class_index as u64) << 16) | (name_and_type_index as u64);
        unsafe {
            std::ptr::write(self.info.offset(index as isize), encoded_value);
        }
    }

    pub fn set_name_and_type(&mut self, index: u16, name_index: u16, descriptor_index: u16) {
        self.tags
            .set(index as JInt, ConstantTag::NameAndType as JByte);
        let encoded_value = ((name_index as u64) << 16) | (descriptor_index as u64);
        unsafe {
            std::ptr::write(self.info.offset(index as isize), encoded_value);
        }
    }

    pub fn set_method_handle(&mut self, index: u16, ref_kind: u8, ref_index: u16) {
        self.tags
            .set(index as JInt, ConstantTag::MethodHandle as JByte);
        let encoded_value = ((ref_kind as u64) << 16) | (ref_index as u64);
        unsafe {
            std::ptr::write(self.info.offset(index as isize), encoded_value);
        }
    }

    pub fn set_method_type(&mut self, index: u16, descriptor_index: u16) {
        self.tags
            .set(index as JInt, ConstantTag::MethodType as JByte);
        unsafe {
            std::ptr::write(self.info.offset(index as isize), descriptor_index as u64);
        }
    }

    pub fn set_invoke_dynamic(
        &mut self,
        index: u16,
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    ) {
        self.tags
            .set(index as JInt, ConstantTag::InvokeDynamic as JByte);
        let encoded_value =
            ((bootstrap_method_attr_index as u64) << 16) | (name_and_type_index as u64);
        unsafe {
            std::ptr::write(self.info.offset(index as isize), encoded_value as u64);
        }
    }
}

pub struct Field {
    // header: Header,
    access_flags: u16,
    name: SymbolPtr,
    descriptor: SymbolPtr,
    constval_index: u16,
}

impl Field {
    const SIZE: usize = size_of::<Self>();

    pub fn new(access_flags: u16, name: SymbolPtr, descriptor: SymbolPtr) -> Field {
        Field {
            access_flags,
            name,
            descriptor,
            constval_index: 0,
        }
        // let mut field = FieldPtr::new(vm::global().heap.alloc(size_of::<Field>()));
        // field.header.initialize(Field::FIELD_CLASS);
        // field.access_flags = access_flags;
        // field.name = name;
        // field.descriptor = descriptor;
        // return field;
    }

    pub fn set_constval_index(&mut self, constval_index: u16) {
        self.constval_index = constval_index;
    }
}

pub struct FieldArray {
    length: u16,
    content: *mut Field,
    // delegate_array: JByteArrayPtr,
}

impl FieldArray {
    // pub const FIELD_ARRAY_CLASS: ClassPtr = ClassPtr::new(ObjectTag::Field as u8 as Address);
    const CONTENT_OFFSET: usize = size_of::<u16>();

    pub fn new(length: u16) -> FieldArrayPtr {
        Ptr::from_addr(vm::instance().heap.alloc_obj_permanent(Self::size(length)))
    }

    pub fn set_field(&mut self, index: u16, field: Field) {
        unsafe {
            let field_ptr = self.content.offset(index as isize);
            *field_ptr = field;
        }
        // TODO refactor typedArray
    }

    pub fn get_field(&self, index: u16) -> FieldPtr {
        unsafe { FieldPtr::new(self.content.offset(index as isize).cast()) }
    }

    pub fn size(length: u16) -> usize {
        Self::CONTENT_OFFSET + length as usize * Field::SIZE
    }
}

pub struct Method {
    // header: Header,
    access_flags: u16,
    name: SymbolPtr,
    descriptor: SymbolPtr,
    max_stack: u16,
    max_locals: u16,
    code: JByteArrayPtr,
}

impl Method {
    // pub const METHOD_CLASS: ClassPtr = ClassPtr::new(ObjectTag::Method as u8 as Address);
    pub const SIZE: usize = size_of::<Method>();

    pub fn new(
        access_flags: u16,
        name: SymbolPtr,
        descriptor: SymbolPtr,
        // max_stack: u16,
        // max_locals: u16,
        // code: JByteArrayPtr,
    ) -> Method {
        Method {
            access_flags,
            name,
            descriptor,
            max_stack: 0,
            max_locals: 0,
            code: JByteArrayPtr::null()
            // max_stack,
            // max_locals,
            // code,
        }
        // let mut method = MethodPtr::new(vm::global().heap.alloc(size_of::<Method>()));
        // method.header.initialize(Method::METHOD_CLASS);
        // method.access_flags = access_flags;
        // method.name = name;
        // method.descriptor = descriptor;
        // method.max_stack = max_stack;
        // method.max_locals = max_locals;
        // method.code = code;
        // return method;
    }

    pub fn set_max_stack(&mut self, max_stack: u16) {
        self.max_stack = max_stack;
    }

    pub fn set_max_locals(&mut self, max_locals: u16) {
        self.max_locals = max_locals;
    }

    pub fn set_code(&mut self, code: JByteArrayPtr) {
        self.code = code;
    }
}

pub struct MethodArray {
    length: u16,
    content: *mut Method,
    // delegate_array: JByteArrayPtr,
}

impl MethodArray {
    const CONTENT_OFFSET: usize = size_of::<u16>();

    pub fn new(length: u16) -> MethodArrayPtr {
        // let methods = JByteArray::new_permanent(length as i32 * Method::SIZE as i32);
        // return MethodArrayPtr::new(methods.as_address());
        JByteArray::new_permanent(length as i32 * Method::SIZE as i32).cast()
    }

    pub fn set_method(&mut self, index: u16, method: Method) {
        unsafe {
            let method_ptr = self.content.offset(Method::SIZE as isize * index as isize);
            *method_ptr = method;
        }
    }

    pub fn get_method(&self, index: u16) -> MethodPtr {
        unsafe { MethodPtr::new(self.content.offset(index as isize).cast()) }
    }

    pub fn size(length: u16) -> usize {
        Self::CONTENT_OFFSET + length as usize * Method::SIZE
    }
}
