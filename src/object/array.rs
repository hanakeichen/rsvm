use super::class::JClassPtr;
use super::prelude::*;
use crate::define_oop;
use crate::memory::align;
use crate::memory::heap::Heap;
use crate::object::class::JClass;
use crate::thread::ThreadPtr;
use crate::vm::VMPtr;
use std::mem::size_of;

pub type JArrayPtr = Ptr<JArray>;

define_oop!(
    struct JArray {
        length: JInt,
        // data: Ptr<u8>,
    }
);

impl JArray {
    pub const DATA_OFFSET: usize = size_of::<JArray>();

    pub fn new(length: JInt, jclass: JClassPtr, thread: ThreadPtr) -> Ptr<JArray> {
        debug_assert!(jclass.class_data().is_array());
        debug_assert!(jclass.class_data().component_type().is_not_null());
        let component_type = jclass.class_data().component_type();
        let size = Self::size(length, JClass::ref_size(component_type));
        let mut array = Ptr::<JArray>::from_addr(Heap::alloc_obj_lab(size, thread));
        array.initialize(length, jclass);
        log::trace!(
            "JArray::new component_type: {}, 0x{:x}, jclass: 0x{:x}",
            component_type.name().as_str(),
            array.as_isize(),
            array.jclass().as_isize()
        );
        return array;
    }

    pub fn new_permanent(length: JInt, jclass: JClassPtr, thread: ThreadPtr) -> Ptr<JArray> {
        debug_assert!(jclass.class_data().is_array());
        debug_assert!(jclass.class_data().component_type().is_not_null());
        let component_type = jclass.class_data().component_type();
        let size = Self::size(length, JClass::ref_size(component_type));
        let mut array = Ptr::<JArray>::from_addr(thread.heap().alloc_obj_permanent(size));
        array.initialize(length, jclass);
        return array;
    }

    pub fn new_internal_permanent(length: JInt, thread: ThreadPtr) -> Ptr<JArray> {
        let jclass = thread.vm().shared_objs().internal_arr_cls;
        debug_assert!(jclass.class_data().is_array());
        debug_assert!(jclass.class_data().component_type().is_not_null());
        let component_type = jclass.class_data().component_type();
        let size = Self::size(length, JClass::ref_size(component_type));
        let mut array = Ptr::<JArray>::from_addr(thread.heap().alloc_obj_permanent(size));
        array.initialize(length, jclass);
        return array;
    }

    pub fn new_obj_arr(length: JInt, thread: ThreadPtr) -> JArrayPtr {
        let classes = thread.vm().preloaded_classes();
        return Self::new(length, classes.jobject_arr_cls(), thread);
    }

    fn initialize(&mut self, length: JInt, jclass: JClassPtr) {
        Object::init_header(JArrayPtr::from_ref(self).cast(), jclass);
        self.length = length;
    }

    const fn size(length: JInt, ref_size: usize) -> usize {
        debug_assert!(length >= 0);
        return align(Self::DATA_OFFSET + ref_size * length as usize);
    }

    pub fn get_component_type(&self) -> JClassPtr {
        self.jclass().class_data().component_type()
    }

    pub fn set(&self, index: JInt, value: ObjectPtr) {
        debug_assert!(index < self.length(), "index out of bound");
        *self.data().offset(index as isize) = value;
    }

    pub fn set_raw(&self, index: JInt, value: ObjectRawPtr) {
        debug_assert!(index < self.length());
        self.set(index, Ptr::from_raw(value));
    }

    pub fn get_raw(&self, index: JInt) -> ObjectRawPtr {
        debug_assert!(index < self.length());
        (*self.data().offset(index as isize)).as_mut_raw_ptr()
    }

    pub fn get(&self, index: JInt) -> ObjectPtr {
        debug_assert!(index < self.length());
        *self.data().offset(index as isize)
    }

    pub fn get_with_isize(&self, index: isize) -> ObjectPtr {
        debug_assert!(index < self.length() as isize);
        *self.data().offset(index)
    }

    pub fn length(&self) -> JInt {
        self.length as JInt
    }

    pub fn data(&self) -> Ptr<ObjectPtr> {
        Ptr::from_ref_offset_bytes(self, Self::DATA_OFFSET as isize)
    }

    pub fn copy_unchecked(
        src: JArrayPtr,
        src_pos: JInt,
        dest: JArrayPtr,
        dest_pos: JInt,
        length: JInt,
    ) {
        unsafe {
            std::ptr::copy(
                src.data().offset(src_pos as isize).as_raw_ptr(),
                dest.data().offset(dest_pos as isize).as_mut_raw_ptr(),
                length as usize,
            );
        }
    }

    pub fn copy_from_raw(&mut self, value: Ptr<ObjectPtr>, length: JInt) {
        unsafe {
            std::ptr::copy(
                value.as_raw_ptr(),
                self.data().as_mut_raw_ptr(),
                length as usize,
            );
        }
    }

    pub fn is_compatible(&self, val: ObjectPtr, vm: VMPtr) -> bool {
        let component_type = self.jclass().class_data().component_type();
        if val.is_null() {
            return !JClass::is_primitive(component_type);
        }
        return component_type.is_assignable_from(val.jclass(), vm);
    }
}

macro_rules! DEFINE_TYPED_ARRAY {
    ($element_type:ident, $array_name:ident, $array_typed_class:expr) => {
        #[derive(Debug)]
        pub struct $array_name {
        }

        #[allow(unused)]
        impl $array_name {

            pub fn size(length: JInt) -> usize {
                assert!(length >= 0);
                return align(JArray::DATA_OFFSET + size_of::<$element_type>() * length as usize);
            }

            pub fn set(&self, index: JInt, value: $element_type) {
                assert!(index < self.length(), "index out of bound");
                unsafe {
                    std::ptr::write(self.data().as_mut_raw_ptr().offset(index as isize), value);
                }
            }

            pub fn get(&self, index: JInt) -> $element_type {
                unsafe {
                    return *self.data().as_mut_raw_ptr().offset(index as isize);
                }
            }

            pub fn length(&self) -> JInt {
                let arr: &JArray = unsafe { std::mem::transmute(self) };
                arr.length as JInt
            }

            pub fn data(&self) -> Ptr<$element_type> {
                Ptr::from_ref_offset_bytes(self, JArray::DATA_OFFSET as isize)
            }

            pub fn to_slice(&self) -> &[$element_type] {
                return unsafe {
                    &*std::ptr::slice_from_raw_parts(
                        self.data().as_raw_ptr() as _,
                        self.length() as usize,
                    )
                };
            }

            pub fn set_length(&mut self, length: JInt) {
                let arr: &mut JArray = unsafe { std::mem::transmute(self) };
                arr.length = length;
            }

            pub fn copy_unchecked(
                src: Ptr<$array_name>,
                src_pos: JInt,
                dest: Ptr<$array_name>,
                dest_pos: JInt,
                length: JInt,
            ) {
                unsafe {
                    std::ptr::copy(
                        src.data().offset(src_pos as isize).as_raw_ptr(),
                        dest.data().offset(dest_pos as isize).as_mut_raw_ptr(),
                        length as usize,
                    );
                }
            }

            pub fn copy_from_raw(&mut self, value: Ptr<$element_type>, length: JInt) {
                unsafe {
                    std::ptr::copy(
                        value.as_raw_ptr(),
                        self.data().as_mut_raw_ptr(),
                        length as usize,
                    );
                }
            }
        }
    };
}

DEFINE_TYPED_ARRAY!(JBoolean, JBooleanArray, global::classes::boolean_class());
DEFINE_TYPED_ARRAY!(JChar, JCharArray, global::classes::char_class());
DEFINE_TYPED_ARRAY!(JByte, JByteArray, global::classes::byte_class());
DEFINE_TYPED_ARRAY!(JShort, JShortArray, global::classes::short_class());
DEFINE_TYPED_ARRAY!(JInt, JIntArray, global::classes::int_class());
DEFINE_TYPED_ARRAY!(JLong, JLongArray, global::classes::long_class());
DEFINE_TYPED_ARRAY!(JFloat, JFloatArray, global::classes::float_class());
DEFINE_TYPED_ARRAY!(JDouble, JDoubleArray, global::classes::double_class());

pub type JCharArrayPtr = Ptr<JCharArray>;
pub type JByteArrayPtr = Ptr<JByteArray>;
pub type JShortArrayPtr = Ptr<JShortArray>;
pub type JIntArrayPtr = Ptr<JIntArray>;
pub type JLongArrayPtr = Ptr<JLongArray>;
pub type JFloatArrayPtr = Ptr<JFloatArray>;
pub type JDoubleArrayPtr = Ptr<JDoubleArray>;
