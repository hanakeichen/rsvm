pub mod array;
pub mod class;
pub mod constant_pool;
pub mod field;
pub mod hash_table;
pub mod method;
pub mod prelude;
mod ptr;
pub mod string;
pub mod symbol;

use field::FieldValue;

use self::{
    array::JArrayPtr,
    class::JClassPtr,
    prelude::{JInt, ObjectPtr, Ptr},
};
use crate::{handle::Handle, memory::heap::Heap, thread::ThreadPtr, vm::VMPtr};

use std::{fmt::Debug, hash::Hasher, mem::size_of};

#[macro_export]
macro_rules! define_oop {
    (struct $obj_name:ident { $($field_name:ident: $field_type:ty,)* }) => {
        #[derive(Debug)]
        pub struct $obj_name {
            header: crate::object::Header,
            $($field_name: $field_type),*
        }

        impl $obj_name {
            #[inline(always)]
            pub fn jclass(&self) -> JClassPtr {
                self.header.jclass
            }

            // #[inline(always)]
            // pub fn set_jclass(&mut self, jclass: JClassPtr) {
            //     self.header.jclass = jclass;
            // }
        }
    };
}

pub trait VMObject {
    fn hash(obj: ObjectPtr) -> JInt;

    fn equals(obj: ObjectPtr, other: ObjectPtr) -> bool;
}

#[derive(Debug)]
struct Header {
    jclass: JClassPtr,
    word: MultiUseWord,
}

impl Header {
    pub const fn size() -> usize {
        size_of::<Header>()
    }
}

#[derive(Debug)]
pub struct Object {
    header: Header,
}

impl Object {
    const FIELDS_OFFSET: usize = std::mem::size_of::<Header>();

    pub fn new(jclass: JClassPtr, thread: ThreadPtr) -> ObjectPtr {
        debug_assert!(jclass.is_initialized());
        let inst_or_ele_size = jclass.class_data().inst_or_ele_size();
        let size = Self::FIELDS_OFFSET + inst_or_ele_size;
        let obj = ObjectPtr::from_addr(Heap::alloc_obj_lab(size, thread));
        Object::init_header(obj, jclass);
        return obj;
    }

    pub fn new_with_hash(jclass: JClassPtr, thread: ThreadPtr, hash: JInt) -> ObjectPtr {
        debug_assert!(jclass.is_initialized());
        let inst_or_ele_size = jclass.class_data().inst_or_ele_size();
        let size = Self::FIELDS_OFFSET + inst_or_ele_size;
        let obj = ObjectPtr::from_addr(Heap::alloc_obj_lab(size, thread));
        Object::init_header_with_hash(obj, jclass, hash);
        return obj;
    }

    pub fn new_permanent(jclass: JClassPtr, thread: ThreadPtr) -> ObjectPtr {
        debug_assert!(jclass.is_initialized());
        let inst_or_ele_size = jclass.class_data().inst_or_ele_size();
        let size = Self::FIELDS_OFFSET + inst_or_ele_size;
        let obj = ObjectPtr::from_addr(thread.heap().alloc_obj_permanent(size));
        Object::init_header(obj, jclass);
        return obj;
    }

    pub fn new_permanent_with_hash(jclass: JClassPtr, thread: ThreadPtr, hash: JInt) -> ObjectPtr {
        debug_assert!(jclass.is_initialized());
        let inst_or_ele_size = jclass.class_data().inst_or_ele_size();
        let size = Self::FIELDS_OFFSET + inst_or_ele_size;
        let obj = ObjectPtr::from_addr(thread.heap().alloc_obj_permanent(size));
        Self::init_header_with_hash(obj, jclass, hash);
        return obj;
    }

    pub fn jclass(&self) -> JClassPtr {
        self.header.jclass
    }

    pub(crate) fn init_header(obj: ObjectPtr, cls: JClassPtr) {
        Self::init_header_with_hash(obj, cls, Self::generate_hash(obj));
    }

    #[inline]
    pub(crate) fn init_header_with_hash(obj: ObjectPtr, cls: JClassPtr, hash: JInt) {
        obj.as_mut_ref().header.jclass = cls;
        obj.as_mut_ref().header.word.set_hash(hash);
    }

    // pub fn set_jclass(&mut self, jclass: JClassPtr) {
    //     self.header.(jclass);
    // }

    pub fn as_ref_array(&self) -> JArrayPtr {
        return JArrayPtr::from_ref(self);
    }

    pub fn is_instance_of(&self, class: JClassPtr, vm: VMPtr) -> bool {
        class.is_assignable_from(self.jclass(), vm)
    }

    pub fn read_value(&self, offset: i32, bytes: i32) -> i64 {
        let dst: Ptr<Self> = Ptr::from_ref_offset_bytes(self, offset as isize);
        if bytes == 1 {
            let dst: Ptr<u8> = dst.cast();
            return (unsafe { std::ptr::read_unaligned(dst.as_raw_ptr()) }) as i64;
        } else if bytes == 2 {
            let dst: Ptr<u16> = dst.cast();
            return (unsafe { std::ptr::read_unaligned(dst.as_raw_ptr()) }) as i64;
        } else if bytes == 4 {
            let dst: Ptr<u32> = dst.cast();
            return (unsafe { std::ptr::read_unaligned(dst.as_raw_ptr()) }) as i64;
        } else if bytes == 8 {
            let dst: Ptr<u64> = dst.cast();
            return (unsafe { std::ptr::read_unaligned(dst.as_raw_ptr()) }) as i64;
        }
        unreachable!();
    }

    pub fn read_value_ptr<T: FieldValue>(&self, offset: isize) -> Ptr<T> {
        let dst: Ptr<Self> = Ptr::from_ref_offset_bytes(self, offset);
        debug_assert!(std::mem::size_of::<T>() <= 8);
        return dst.cast();
    }

    pub fn hash(&self) -> JInt {
        return self.header.word.hash();
    }

    pub fn clone(src: ObjectPtr, thread: ThreadPtr) -> Handle<Object> {
        let jclass = src.jclass();
        let result = Handle::new(Self::new(jclass, thread));
        let obj = result.as_ptr();
        let inst_or_ele_size = jclass.class_data().inst_or_ele_size();
        unsafe {
            std::ptr::copy(
                src.as_address()
                    .offset(Self::FIELDS_OFFSET as isize)
                    .raw_ptr(),
                obj.as_address()
                    .offset(Self::FIELDS_OFFSET as isize)
                    .as_mut_raw_ptr(),
                inst_or_ele_size,
            )
        };
        return result;
    }

    fn generate_hash(obj: ObjectPtr) -> JInt {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        hasher.write_isize(obj.as_isize());
        return hasher.finish() as JInt;
    }
}

#[derive(Debug)]
struct MultiUseWord {
    value: MultiUseWordValue,
}

impl MultiUseWord {
    fn hash(&self) -> JInt {
        return unsafe { self.value.h.1 };
    }

    fn set_hash(&mut self, hash: JInt) {
        // self.value &= 0xffffffffu64;
        // self.value |= (hash as u64) << 32;
        self.value.h.1 = hash;
    }
}

union MultiUseWordValue {
    l: u64,
    h: (JInt, JInt),
}

impl Debug for MultiUseWordValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:x}", unsafe { self.l }))
    }
}
