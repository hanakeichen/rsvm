use super::prelude::*;
use super::Header;
// use crate::runtime::{BootstrapClassId, Runtime};
use crate::global;
use crate::vm;
use std::mem::size_of;

macro_rules! DEFINE_TYPED_ARRAY {
    ($element_type:ident, $array_name:ident, $array_typed_class:expr) => {
        pub struct $array_name {
            header: Header,
            length: JInt,
            data: Ptr<$element_type>,
        }

        impl $array_name {
            pub fn new(length: JInt) -> Ptr<$array_name> {
                let size = Self::object_size(length);
                let mut array = Ptr::<$array_name>::from_addr(vm::instance().heap.alloc_obj(size));
                array.initialize(length);
                return array;
            }

            pub fn new_permanent(length: JInt) -> Ptr<$array_name> {
                let size = Self::object_size(length);
                let mut array =
                    Ptr::<$array_name>::from_addr(vm::instance().heap.alloc_obj_permanent(size));
                array.initialize(length);
                return array;
            }

            fn initialize(&mut self, length: JInt) {
                self.header.initialize($array_typed_class);
                self.length = length;
                let data_ptr = self.data.as_mut_ptr();
                for i in 0..length {
                    unsafe {
                        std::ptr::write(data_ptr.offset(i as isize), $element_type::default());
                    }
                }
            }

            fn object_size(length: JInt) -> usize {
                assert!(length >= 0);
                return size_of::<Header>()
                    + size_of::<JInt>()
                    + size_of::<$element_type>() * length as usize;
            }

            pub fn set(&self, index: JInt, value: $element_type) {
                assert!(index < self.length, "index out of bound");
                unsafe {
                    std::ptr::write(self.data.as_mut_ptr().offset(index as isize), value);
                }
            }

            pub fn get(&self, index: JInt) -> $element_type {
                unsafe {
                    return *self.data.as_mut_ptr().offset(index as isize);
                }
            }

            pub fn length(&self) -> JInt {
                self.length
            }

            pub fn raw_data(&self) -> Ptr<$element_type> {
                self.data
            }
        }
    };
}

DEFINE_TYPED_ARRAY!(JBoolean, JBooleanArray, global::classes::cclass());
DEFINE_TYPED_ARRAY!(JChar, JCharArray, global::classes::char_class());
DEFINE_TYPED_ARRAY!(JByte, JByteArray, global::classes::byte_class());
DEFINE_TYPED_ARRAY!(JShort, JShortArray, global::classes::short_class());
DEFINE_TYPED_ARRAY!(JInt, JIntArray, global::classes::int_class());
DEFINE_TYPED_ARRAY!(JLong, JLongArray, global::classes::long_class());
DEFINE_TYPED_ARRAY!(JFloat, JFloatArray, global::classes::float_class());
DEFINE_TYPED_ARRAY!(JDouble, JDoubleArray, global::classes::double_class());

pub type JBooleanArrayPtr = Ptr<JBooleanArray>;
pub type JCharArrayPtr = Ptr<JCharArray>;
pub type JByteArrayPtr = Ptr<JByteArray>;
pub type JShortArrayPtr = Ptr<JShortArray>;
pub type JIntArrayPtr = Ptr<JIntArray>;
pub type JLongArrayPtr = Ptr<JLongArray>;
pub type JFloatArrayPtr = Ptr<JFloatArray>;
pub type JDoubleArrayPtr = Ptr<JDoubleArray>;
pub type JRefArrayPtr = Ptr<JRefArray>;

pub struct MultiArrayClass {
    header: Header,
    name: SymbolPtr,
    dimension: JInt,
}

impl MultiArrayClass {
    pub fn new(array_class: ClassPtr, dimension: JInt) -> Ptr<Self> {
        let mut result: Ptr<Self> =
            Ptr::from_addr(vm::instance().heap.alloc_obj_permanent(Self::size()));
        result.header.initialize(global::classes::cclass());
        result.name = array_class.name();
        result.dimension = dimension;
        return result;
    }

    pub fn size() -> usize {
        size_of::<MultiArrayClass>()
    }
}

pub struct MultiArray {
    header: Header,
    length: JInt,
    data: Ptr<ObjectPtr>,
}

impl MultiArray {
    pub fn new(array_class: ClassPtr, dimension: JInt, length: JInt) -> Ptr<MultiArray> {
        assert!(dimension > 1);
        let mut result: Ptr<MultiArray> = Ptr::from_addr(
            vm::instance()
                .heap
                .alloc_obj_permanent(Self::size(dimension, length)),
        );
        result
            .header
            .initialize(MultiArrayClass::new(array_class, dimension).cast());
        result.length = length;
        for i in 0..dimension - 1 {
            // TODO initialize dimensionn array
        }
        return result;
    }

    pub fn size(dimension: JInt, length: JInt) -> usize {
        Header::size() + size_of::<JInt>() + size_of::<ObjectPtr>() * length as usize
    }
}

pub struct JRefArray {
    header: Header,
    length: JInt,
    data: Ptr<ObjectPtr>,
}

impl JRefArray {
    pub fn new(length: JInt, class: ClassPtr) -> JRefArrayPtr {
        let size = Self::object_size(length);
        let mut array = JRefArrayPtr::from_addr(vm::instance().heap.alloc_obj(size));
        array.initialize(length, class);
        return array;
    }

    pub fn new_permanent(length: JInt, class: ClassPtr) -> JRefArrayPtr {
        let size = Self::object_size(length);
        let mut array = JRefArrayPtr::from_addr(vm::instance().heap.alloc_obj_permanent(size));
        array.initialize(length, class);
        return array;
    }

    pub fn new_obj_permanent(length: JInt) -> JRefArrayPtr {
        Self::new_permanent(length, global::classes::obj_class())
    }

    fn initialize(&mut self, length: JInt, class: ClassPtr) {
        self.header.initialize(class);
        self.length = length;
        let data_ptr = self.data.as_mut_ptr();
        for i in 0..length {
            unsafe {
                std::ptr::write(data_ptr.offset(i as isize), ObjectPtr::null());
            }
        }
    }

    fn object_size(length: JInt) -> usize {
        assert!(length >= 0);
        return size_of::<Header>() + size_of::<JInt>() + size_of::<ObjectPtr>() * length as usize;
    }

    pub fn set(&self, index: JInt, value: ObjectPtr) {
        assert!(index < self.length, "index out of bound");
        assert!(value.is_jobject());
        unsafe { std::ptr::write(self.data.as_mut_ptr().offset(index as isize), value) }
    }

    pub fn get(&self, index: JInt) -> ObjectPtr {
        unsafe {
            return *self.data.as_mut_ptr().offset(index as isize);
        }
    }

    pub fn length(&self) -> JInt {
        self.length
    }
}
