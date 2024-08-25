use std::mem::size_of;

use crate::{define_oop, memory::align, thread::ThreadPtr};

use super::{
    array::{JByteArray, JByteArrayPtr},
    class::JClassPtr,
    prelude::{JByte, JDouble, JFloat, JInt, JLong},
    ptr::Ptr,
    string::VMStringPtr,
    symbol::SymbolPtr,
    Header, Object,
};

pub type ConstantPoolPtr = Ptr<ConstantPool>;

pub struct ConstMemberRef {
    pub class_name: SymbolPtr,
    pub member_name: SymbolPtr,
    pub member_desc: SymbolPtr,
}

impl ConstMemberRef {
    pub fn new(class_name: SymbolPtr, member_name: SymbolPtr, member_desc: SymbolPtr) -> Self {
        Self {
            class_name,
            member_name,
            member_desc,
        }
    }
}

#[derive(Debug)]
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

define_oop!(
    struct ConstantPool {}
);

impl ConstantPool {
    pub fn new(length: u16, thread: ThreadPtr) -> ConstantPoolPtr {
        let size = ConstantPool::size(length);
        let cp = ConstantPoolPtr::from_addr(thread.heap().alloc_code(size));
        Object::init_header(cp.cast(), thread.vm().shared_objs().internal_cp_cls);
        cp.set_tags_length(length);
        return cp;
    }

    pub fn length(&self) -> u16 {
        self.tags().length() as u16
    }

    pub fn get_tag(&self, index: u16) -> ConstantTag {
        unsafe { ::std::mem::transmute(self.tags().get(index as i32)) }
    }

    pub fn set_invalid(&self, index: u16) {
        self.tags()
            .set(index as JInt, ConstantTag::Invalid as JByte);
    }

    pub fn set_utf8(&mut self, index: u16, value: SymbolPtr) {
        self.tags().set(index as JInt, ConstantTag::Utf8 as JByte);
        unsafe {
            std::ptr::write(
                self.raw_info().offset(index as isize),
                value.as_usize() as u64,
            );
        }
    }

    pub fn get_utf8(&self, index: u16) -> SymbolPtr {
        debug_assert_eq!(self.tags().get(index as JInt), ConstantTag::Utf8 as JByte);
        unsafe {
            let addr = std::ptr::read(self.raw_info().offset(index as isize)) as usize;
            return SymbolPtr::from_usize(addr);
        }
    }

    pub fn get_int32(&self, index: u16) -> JInt {
        debug_assert_eq!(
            self.tags().get(index as JInt),
            ConstantTag::Integer as JByte
        );
        unsafe {
            let val = std::ptr::read(self.raw_info().offset(index as isize));
            return std::mem::transmute_copy(&val);
        }
    }

    pub fn set_int32(&mut self, index: u16, value: JInt) {
        self.tags()
            .set(index as JInt, ConstantTag::Integer as JByte);
        unsafe {
            std::ptr::write(self.raw_info().offset(index as isize), value as u64);
        }
    }

    pub fn get_float(&self, index: u16) -> JFloat {
        debug_assert_eq!(self.tags().get(index as JInt), ConstantTag::Float as JByte);
        unsafe {
            let val = std::ptr::read(self.raw_info().offset(index as isize));
            return std::mem::transmute_copy(&val);
        }
    }

    pub fn set_float(&mut self, index: u16, value: JFloat) {
        self.tags().set(index as JInt, ConstantTag::Float as JByte);
        unsafe {
            std::ptr::write(self.raw_info().offset(index as isize), value as u64);
        }
    }

    pub fn get_long(&self, index: u16) -> JLong {
        debug_assert_eq!(self.tags().get(index as JInt), ConstantTag::Long as JByte);
        unsafe {
            let val = std::ptr::read(self.raw_info().offset(index as isize));
            return std::mem::transmute(val);
        }
    }

    pub fn set_long(&mut self, index: u16, value: JLong) {
        self.tags().set(index as JInt, ConstantTag::Long as JByte);
        unsafe {
            std::ptr::write(self.raw_info().offset(index as isize), value as u64);
        }
    }

    pub fn get_double(&self, index: u16) -> JDouble {
        debug_assert_eq!(self.tags().get(index as JInt), ConstantTag::Double as JByte);
        unsafe {
            let val = std::ptr::read(self.raw_info().offset(index as isize));
            return std::mem::transmute(val);
        }
    }

    pub fn set_double(&mut self, index: u16, value: JDouble) {
        self.tags().set(index as JInt, ConstantTag::Double as JByte);
        unsafe {
            std::ptr::write(self.raw_info().offset(index as isize), value as u64);
        }
    }

    pub fn set_class_index(&mut self, index: u16, class_index: u16) {
        self.tags().set(index as JInt, ConstantTag::Class as JByte);
        unsafe {
            std::ptr::write(self.raw_info().offset(index as isize), class_index as u64);
        }
    }

    pub fn get_class_name(&self, index: u16) -> SymbolPtr {
        let index_tag = self.tags().get(index as i32);
        assert_eq!(index_tag, ConstantTag::Class as JByte);
        unsafe {
            let name_index = std::ptr::read(self.raw_info().offset(index as isize)) as u16;
            assert!((name_index as i32) < self.tags().length());
            let name_index_tag = self.tags().get(name_index as JInt);
            if name_index_tag == ConstantTag::Utf8 as JByte {
                return self.get_utf8(name_index);
            }
            let (name, _) = self.get_name_type_info(name_index);
            return name;
        }
    }

    pub fn get_name_type_info(&self, index: u16) -> (SymbolPtr, SymbolPtr) {
        let index_tag = self.tags().get(index as i32);
        assert_eq!(index_tag, ConstantTag::NameAndType as JByte);
        unsafe {
            let encoded_name_type = std::ptr::read(self.raw_info().offset(index as isize));
            let name_index = ((encoded_name_type >> 16) & 0xffff) as u16;
            let desc_index = (encoded_name_type & 0xffff) as u16;
            return (self.get_utf8(name_index), self.get_utf8(desc_index));
        }
    }

    pub fn get_string(&self, index: u16) -> VMStringPtr {
        debug_assert_eq!(self.tags().get(index as JInt), ConstantTag::String as JByte);
        unsafe {
            let string_index = std::ptr::read(self.raw_info().offset(index as isize));
            return self.get_utf8(string_index as u16);
        }
    }

    pub fn set_string(&mut self, index: u16, string_index: u16) {
        self.tags().set(index as JInt, ConstantTag::String as JByte);
        unsafe {
            std::ptr::write(self.raw_info().offset(index as isize), string_index as u64);
        }
    }

    pub fn set_field_ref(&mut self, index: u16, class_index: u16, name_and_type_index: u16) {
        self.tags()
            .set(index as JInt, ConstantTag::Fieldref as JByte);
        let encoded_value = ((class_index as u64) << 16) | (name_and_type_index as u64);
        unsafe {
            std::ptr::write(self.raw_info().offset(index as isize), encoded_value);
        }
    }

    pub fn get_field_ref(&self, index: u16) -> ConstMemberRef {
        debug_assert_eq!(
            self.tags().get(index as JInt),
            ConstantTag::Fieldref as JByte
        );
        return self.get_member_ref(index);
    }

    pub fn get_method_ref(&self, index: u16) -> ConstMemberRef {
        debug_assert_eq!(
            self.tags().get(index as JInt),
            ConstantTag::Methodref as JByte
        );
        return self.get_member_ref(index);
    }

    pub fn set_method_ref(&mut self, index: u16, class_index: u16, name_and_type_index: u16) {
        self.tags()
            .set(index as JInt, ConstantTag::Methodref as JByte);
        let encoded_value = ((class_index as u64) << 16) | (name_and_type_index as u64);
        unsafe {
            std::ptr::write(self.raw_info().offset(index as isize), encoded_value);
        }
    }

    pub fn get_interface_method_ref(&self, index: u16) -> ConstMemberRef {
        debug_assert_eq!(
            self.tags().get(index as JInt),
            ConstantTag::InterfaceMethodref as JByte
        );
        return self.get_member_ref(index);
    }

    pub fn set_interface_method_ref(
        &mut self,
        index: u16,
        class_index: u16,
        name_and_type_index: u16,
    ) {
        self.tags()
            .set(index as JInt, ConstantTag::InterfaceMethodref as JByte);
        let encoded_value = ((class_index as u64) << 16) | (name_and_type_index as u64);
        unsafe {
            std::ptr::write(self.raw_info().offset(index as isize), encoded_value);
        }
    }

    pub fn set_name_and_type(&mut self, index: u16, name_index: u16, descriptor_index: u16) {
        self.tags()
            .set(index as JInt, ConstantTag::NameAndType as JByte);
        let encoded_value = ((name_index as u64) << 16) | (descriptor_index as u64);
        unsafe {
            std::ptr::write(self.raw_info().offset(index as isize), encoded_value);
        }
    }

    pub fn set_method_handle(&mut self, index: u16, ref_kind: u8, ref_index: u16) {
        self.tags()
            .set(index as JInt, ConstantTag::MethodHandle as JByte);
        let encoded_value = ((ref_kind as u64) << 16) | (ref_index as u64);
        unsafe {
            std::ptr::write(self.raw_info().offset(index as isize), encoded_value);
        }
    }

    pub fn set_method_type(&mut self, index: u16, descriptor_index: u16) {
        self.tags()
            .set(index as JInt, ConstantTag::MethodType as JByte);
        unsafe {
            std::ptr::write(
                self.raw_info().offset(index as isize),
                descriptor_index as u64,
            );
        }
    }

    pub fn set_invoke_dynamic(
        &mut self,
        index: u16,
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    ) {
        self.tags()
            .set(index as JInt, ConstantTag::InvokeDynamic as JByte);
        let encoded_value =
            ((bootstrap_method_attr_index as u64) << 16) | (name_and_type_index as u64);
        unsafe {
            std::ptr::write(self.raw_info().offset(index as isize), encoded_value as u64);
        }
    }

    pub fn info(&self) -> Ptr<u64> {
        return Ptr::from_self_offset_bytes::<u64>(self, self.raw_info_offset() as isize);
    }

    fn raw_info(&self) -> *mut u64 {
        return Ptr::from_self_offset_bytes::<u64>(self, self.raw_info_offset() as isize)
            .as_mut_raw_ptr();
    }

    fn size(length: u16) -> usize {
        align(Header::size() + JByteArray::size(length as i32) + length as usize * size_of::<u64>())
    }

    fn tags(&self) -> JByteArrayPtr {
        return Ptr::from_self_offset_bytes::<JByteArray>(self, Header::size() as isize);
    }

    fn get_member_ref(&self, index: u16) -> ConstMemberRef {
        let field_ref;
        unsafe {
            field_ref = *self.raw_info().offset(index as isize);
        }
        let class_index = (field_ref >> 16) as u16;
        let name_and_type_index = (field_ref & 0xffff) as u16;
        let class_name = self.get_class_name(class_index);
        let (member_name, member_desc) = self.get_name_type_info(name_and_type_index);
        return ConstMemberRef {
            class_name,
            member_name,
            member_desc,
        };
    }

    fn set_tags_length(&self, length: u16) {
        let mut tags = self.tags();
        tags.set_length(length as i32);
    }

    fn raw_info_offset(&self) -> usize {
        return Header::size() + JByteArray::size(self.tags().length());
    }
}
