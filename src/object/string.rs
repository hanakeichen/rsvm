use std::{
    hash::{Hash, Hasher},
    mem::size_of,
};

use crate::{
    define_oop,
    memory::{align, Address},
    object::Object,
    thread::{Thread, ThreadPtr},
    vm::VM,
    ObjectPtr,
};

use super::{
    array::JCharArrayPtr,
    class::JClassPtr,
    prelude::{JChar, JInt, Ptr},
    Header, VMObject,
};

pub type HeapStringPtr = Ptr<HeapString>;
pub type VMStringPtr = HeapStringPtr;
pub type JStringPtr = Ptr<JString>;
pub type Utf16String = Vec<u16>;

#[derive(Copy, Clone)]
pub struct Utf8String<'a> {
    pub value: &'a str,
}

impl<'a> From<&'a str> for Utf8String<'a> {
    fn from(value: &'a str) -> Self {
        return Utf8String { value };
    }
}

impl<'a> Utf8String<'a> {
    pub fn hash(&self) -> i32 {
        return HeapString::hash_utf8(self.value);
    }
}

define_oop!(
    struct HeapString {
        hash: JInt,
        length: JInt,
    }
);

impl HeapString {
    const BYTES_OFFSET: usize = Header::size() + size_of::<isize>();

    pub fn new(content: &str, thread: ThreadPtr) -> HeapStringPtr {
        return Self::new_with_hash(content, Self::hash_utf8(&content), thread);
    }

    pub fn new_with_hash(content: &str, hash: JInt, thread: ThreadPtr) -> HeapStringPtr {
        let mut vm_str = HeapStringPtr::from_addr(
            thread
                .heap()
                .alloc_obj_permanent(Self::obj_size(content.len())),
        );
        let cls = thread.vm().shared_objs().vm_str_cls;
        vm_str.hash = hash;
        vm_str.length = content.len() as JInt;
        vm_str.set_bytes(content.as_ptr());
        Object::init_header_with_hash(vm_str.cast(), cls, hash);
        debug_assert_eq!(vm_str.as_str(), content);
        return vm_str;
    }

    pub fn hash_utf8(content: &str) -> JInt {
        let mut hash: JInt = 0;
        for ch in content.chars() {
            hash = hash ^ ch as JInt;
            hash = hash * 0x01000193;
        }
        return hash;
    }

    pub fn hash_utf16_ptr(content: Ptr<u16>, length: JInt) -> JInt {
        let mut hash: JInt = 0;
        let content = content.as_slice(length as usize);
        for ch in content {
            hash = hash ^ *ch as JInt;
            hash = hash * 0x01000193;
        }
        return hash;
    }

    pub fn hash_utf16_str(utf16_str: &Utf16String) -> JInt {
        return Self::hash_utf16_ptr(Ptr::from_raw(utf16_str.as_ptr()), utf16_str.len() as JInt);
    }

    pub fn length(&self) -> JInt {
        self.length
    }

    pub fn hash_code(&self) -> JInt {
        self.hash
    }

    pub fn as_bytes<'a>(&self) -> &'a [u8] {
        unsafe {
            return core::slice::from_raw_parts(
                self.bytes_ptr().as_raw_ptr(),
                self.length as usize,
            );
        }
    }

    pub fn as_str(&self) -> &str {
        debug_assert!(
            self.jclass().is_null()
                || self.jclass() == Thread::current().vm().shared_objs().vm_str_cls
        );
        return unsafe { std::str::from_utf8_unchecked(self.as_bytes()) };
    }

    pub fn to_utf16(&self) -> Vec<u16> {
        return self.as_str().encode_utf16().collect();
    }

    pub fn equals_utf8(&self, ref_str: Utf8String) -> bool {
        return self.as_str() == ref_str.value;
    }

    pub fn equals_utf16_ptr(&self, target: Ptr<i16>, target_len: JInt) -> bool {
        let target = target.as_slice(target_len as usize);
        let mut src_idx = 0;
        for ch in self.as_str().chars() {
            if src_idx == target_len {
                return false;
            }
            if target[src_idx as usize] as JInt != ch as JInt {
                return false;
            }
            src_idx += 1;
        }
        return src_idx == target_len;
    }

    pub fn debug(&self) {
        Address::from_ref(self);
        let i64_ptr: Ptr<i64> = Ptr::from_ref(self);
        log::trace!(
            "offset 0: {} i64, offset 8: {} i64, offset 16: {} i32",
            *i64_ptr,
            *i64_ptr.offset(1),
            *i64_ptr.offset(2).cast::<i32>()
        );
    }

    fn bytes_ptr(&self) -> Ptr<u8> {
        Ptr::from_ref_offset_bytes(self, Self::BYTES_OFFSET as isize)
    }

    fn set_bytes(&self, bytes: *const u8) {
        unsafe {
            std::ptr::copy(
                bytes,
                self.bytes_ptr().as_mut_raw_ptr(),
                self.length as usize,
            );
        }
    }

    const fn obj_size(length: usize) -> usize {
        return align(Self::BYTES_OFFSET + size_of::<u16>() * length);
    }
}

impl VMObject for HeapString {
    fn hash(obj: ObjectPtr) -> JInt {
        return obj.cast::<HeapString>().hash;
    }

    fn equals(obj: ObjectPtr, other: ObjectPtr) -> bool {
        if obj == other {
            return true;
        }
        if obj.jclass() != other.jclass() {
            return false;
        }
        return obj.cast::<HeapString>().as_str() == other.cast::<HeapString>().as_str();
    }
}

impl Hash for HeapString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i32(self.hash);
        state.finish();
    }
}

impl PartialEq for HeapString {
    fn eq(&self, other: &Self) -> bool {
        Ptr::from_self(self).as_usize() == Ptr::from_self(other).as_usize()
    }
}

impl Hash for HeapStringPtr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i32(self.hash);
        state.finish();
    }
}

pub struct JString;

impl JString {
    pub fn get_char_array(jstr: JStringPtr, vm: &VM) -> JCharArrayPtr {
        return vm
            .shared_objs()
            .class_infos()
            .java_lang_string_info()
            .get_chars(jstr);
    }

    pub fn get_chars(jstr: JStringPtr, vm: &VM) -> Ptr<i16> {
        return vm
            .shared_objs()
            .class_infos()
            .java_lang_string_info()
            .get_chars(jstr)
            .data();
    }

    pub fn to_rust_string(jstr: JStringPtr, vm: &VM) -> String {
        let chars = Self::get_char_array(jstr, vm);
        let chars = chars.to_slice();
        let chars: &[u16] = unsafe { std::mem::transmute(chars) };
        return String::from_utf16_lossy(chars);
    }

    pub fn str_to_utf16(val: &str) -> Utf16String {
        return val.encode_utf16().collect();
    }

    pub fn char_arr_set_utf16_unchecked(
        char_arr: JCharArrayPtr,
        utf16_str: &Vec<u16>,
        utf16_len: JInt,
    ) {
        char_arr
            .as_mut_ref()
            .copy_from_raw(Ptr::from_raw(utf16_str.as_ptr() as *const i16), utf16_len);
    }

    pub fn equals_utf16(jstr: JStringPtr, utf16_str: &Utf16String, vm: &VM) -> bool {
        let jstr_chars = Self::get_char_array(jstr, vm);
        let jstr_chars_len = jstr_chars.length();
        if jstr_chars_len != utf16_str.len() as JInt {
            return false;
        }
        for idx in 0..jstr_chars_len {
            if jstr_chars.get(idx) != unsafe { *utf16_str.get_unchecked(idx as usize) } as JChar {
                return false;
            }
        }
        return true;
    }

    pub fn equals_chars(chars1: JCharArrayPtr, chars2: JCharArrayPtr) -> bool {
        let length = chars1.length();
        if length != chars2.length() {
            return false;
        }
        for idx in 0..length {
            if chars1.get(idx) != chars2.get(idx) {
                return false;
            }
        }
        return true;
    }
}

impl VMObject for JString {
    fn hash(obj: ObjectPtr) -> JInt {
        return obj.hash();
    }

    fn equals(obj: ObjectPtr, other: ObjectPtr) -> bool {
        if obj == other {
            return true;
        }
        if obj.jclass() != other.jclass() {
            return false;
        }
        let thread = Thread::current();
        let string_info = thread
            .vm()
            .shared_objs()
            .class_infos()
            .java_lang_string_info();
        return string_info.get_chars(obj.cast()) == string_info.get_chars(other.cast());
    }
}
