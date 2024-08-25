use std::mem::size_of;

use crate::{
    classfile::ClassLoadErr, define_oop, memory::align, object::{class::JClass, Object}, thread::ThreadPtr,
    JClassPtr, ObjectPtr,
};

use super::{
    array::{JArrayPtr, JCharArrayPtr},
    prelude::{JByte, JChar, JDouble, JFloat, JInt, JLong, ObjectRawPtr},
    ptr::Ptr,
    string::JStringPtr,
    symbol::SymbolPtr,
};

pub type FieldPtr = Ptr<Field>;

define_oop!(
    struct Field {
        access_flags: u16,
        layout_offset: u16,
        name: SymbolPtr,
        _field_class_or_null: JClassPtr,
        descriptor: SymbolPtr,
        constval_index: u16,
    }
);

impl Field {
    pub fn new(
        access_flags: u16,
        layout_offset: u16,
        name: SymbolPtr,
        descriptor: SymbolPtr,
        field_class_or_null: JClassPtr,
        thread: ThreadPtr,
    ) -> FieldPtr {
        let mut field = FieldPtr::from_addr(thread.vm().heap().alloc_code(Self::size()));
        field.access_flags = access_flags;
        field.layout_offset = layout_offset;
        field.name = name;
        field.descriptor = descriptor;
        field._field_class_or_null = field_class_or_null;
        field.constval_index = 0;
        return field;
        // let mut field = FieldPtr::new(vm::global().heap.alloc(size_of::<Field>()));
        // field.header.initialize(Field::FIELD_CLASS);
        // field.access_flags = access_flags;
        // field.name = name;
        // field.descriptor = descriptor;
        // return field;
    }

    pub fn name(&self) -> SymbolPtr {
        self.name
    }

    pub fn descriptor(&self) -> SymbolPtr {
        self.descriptor
    }

    pub fn access_flags(&self) -> u16 {
        self.access_flags
    }

    pub fn is_static(&self) -> bool {
        return FieldAccessFlags::is_static(self.access_flags);
    }

    pub fn set_constval_index(&mut self, constval_index: u16) {
        self.constval_index = constval_index;
    }

    pub fn get_value(&self, obj: ObjectPtr, thread: ThreadPtr) -> Result<i64, ClassLoadErr> {
        debug_assert!(!self.is_static());

        return Ok(obj.read_value(
            self.layout_offset as i32,
            JClass::ref_size(self.field_class(thread)?) as i32,
        ));
    }

    pub fn get_typed_value<T: Copy + FieldValue>(&self, obj: ObjectPtr) -> T {
        debug_assert!(!self.is_static());

        return *obj.read_value_ptr(self.layout_offset as isize);
    }

    pub fn fast_get_value(&self, obj: ObjectPtr) -> i64 {
        debug_assert!(!self.is_static());
        debug_assert!(self._field_class_or_null.is_not_null());

        return obj.read_value(
            self.layout_offset as i32,
            JClass::ref_size(self._field_class_or_null) as i32,
        );
    }

    pub fn set_typed_value<T: FieldValue>(&self, obj: ObjectPtr, value: T) {
        let fields_addr = obj.as_address();
        let field_ptr: Ptr<T> = Ptr::from_addr(fields_addr.offset(self.layout_offset as isize));
        unsafe {
            std::ptr::write_unaligned(field_ptr.as_mut_raw_ptr(), value);
        }
        // *field_ptr = value;
    }

    pub fn get_static_value(&self, class: JClassPtr) -> i64 {
        debug_assert!(self.is_static());
        return class.get_static_value(
            self.layout_offset as i32,
            JClass::ref_size(self._field_class_or_null) as i32,
        );
    }

    pub fn get_static_typed_value<T: FieldValue>(&self, class: JClassPtr) -> T {
        debug_assert!(self.is_static());
        return *class.cast::<Object>().read_value_ptr(self.layout_offset as isize);
    }

    pub fn set_static_value<T: FieldValue>(&self, class: JClassPtr, val: T) {
        return class.set_static_value(self.layout_offset as i32, val);
    }

    pub fn instance_size(&self) -> usize {
        return JClass::ref_size(self._field_class_or_null);
        // if self.decl_class().is_primitive() {
        //     return self.decl_class.ref_size();
        // }
        // return self.decl_class.ref_size();
    }

    pub fn layout_offset(&self) -> u16 {
        self.layout_offset
    }

    pub fn set_layout_offset(&mut self, layout_offset: u16) {
        self.layout_offset = layout_offset;
    }

    pub fn field_class_is_primitive(&self) -> bool {
        return self._field_class_or_null.is_not_null()
            && self._field_class_or_null.class_data().is_primitive();
    }

    pub fn field_class_unchecked(&self) -> JClassPtr {
        return self._field_class_or_null;
    }

    pub fn field_class(&self, thread: ThreadPtr) -> Result<JClassPtr, ClassLoadErr> {
        if self._field_class_or_null.is_null() {
            // TODO
            let mut self_ptr = FieldPtr::from_ref(self);
            let field_class = thread
                .vm()
                .bootstrap_class_loader
                .load_class(self.descriptor.as_str())?;
            self_ptr._field_class_or_null = field_class;
            return Ok(field_class);
        }
        return Ok(self._field_class_or_null);
    }

    pub fn set_field_class(&mut self, field_class: JClassPtr) {
        self._field_class_or_null = field_class;
    }

    const fn size() -> usize {
        return align(size_of::<Self>());
    }
}

#[allow(unused)]
pub enum FieldAccessFlags {
    AccPublic = 0x0001,
    AccPrivate = 0x0002,
    AccProtected = 0x0004,
    AccStatic = 0x0008,
    AccFinal = 0x0010,
    AccVolatile = 0x0040,
    AccTransient = 0x0080,
}

impl FieldAccessFlags {
    pub fn is_static(access_flags: u16) -> bool {
        return access_flags & FieldAccessFlags::AccStatic as u16 != 0;
    }
}

pub trait FieldValue : Copy {}

impl FieldValue for JByte {}
impl FieldValue for JChar {}
impl FieldValue for JInt {}
impl FieldValue for JFloat {}
impl FieldValue for JLong {}
impl FieldValue for JDouble {}
impl FieldValue for ObjectPtr {}
impl FieldValue for ObjectRawPtr {}
impl FieldValue for JClassPtr {}
impl FieldValue for JArrayPtr {}
impl FieldValue for JCharArrayPtr {}
impl FieldValue for JStringPtr {}
