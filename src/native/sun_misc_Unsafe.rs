use std::{
    alloc::Layout,
    sync::atomic::{AtomicI32, AtomicPtr, Ordering},
};

use jni::{
    objects::{JClass, JObject},
    sys::{jboolean, jbyte, jint, jlong},
    JNIEnv,
};

use crate::{
    memory::{align, POINTER_SIZE},
    object::prelude::{JInt, Ptr},
    JClassPtr, ObjectPtr,
};

use super::jni::JNIEnvWrapper;

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_sun_misc_Unsafe_registerNatives<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
) {
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_sun_misc_Unsafe_getByte<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
    address: jlong,
) -> jbyte {
    unsafe { *(address as *mut jbyte) }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_sun_misc_Unsafe_putLong<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
    address: jlong,
    x: jlong,
) {
    unsafe {
        *(address as *mut jlong) = x;
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_sun_misc_Unsafe_allocateMemory<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
    bytes: jlong,
) -> jlong {
    if bytes < 0 {
        todo!("throw IllegalArgumentException");
    }
    unsafe {
        let layout = Layout::from_size_align_unchecked(
            std::mem::size_of::<Layout>() + align(bytes as usize),
            POINTER_SIZE,
        );
        let bytes = std::alloc::alloc(layout);
        if bytes.is_null() {
            todo!("throw OutOfMemoryError");
        }
        *(bytes as *mut Layout) = layout;
        bytes.offset(std::mem::size_of::<Layout>() as isize) as jlong
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_sun_misc_Unsafe_freeMemory<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
    address: jlong,
) {
    unsafe {
        let address = (address as *mut u8).offset(-(std::mem::size_of::<Layout>() as isize));
        let layout = *(address as *const Layout);
        std::alloc::dealloc(address, layout);
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_sun_misc_Unsafe_objectFieldOffset<'local>(
    env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
    field: JObject<'local>,
) -> jlong {
    let vm = JNIEnvWrapper::from_raw_env(env.get_raw()).vm();
    let slot_field = vm
        .shared_objs()
        .class_infos()
        .java_lang_reflect_field_info()
        .slot_field();
    let field_obj = ObjectPtr::from_raw(field.as_raw() as _);
    let offset: jint = slot_field.get_typed_value(field_obj);
    log::trace!(
        "Java_sun_misc_Unsafe_objectFieldOffset offset: {}, {}",
        offset,
        offset as jlong
    );
    return offset as jlong;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_sun_misc_Unsafe_arrayBaseOffset<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
    _arr_cls: JObject<'local>,
) -> jint {
    return crate::JArray::DATA_OFFSET as jint;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_sun_misc_Unsafe_arrayIndexScale<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
    arr_cls: JObject<'local>,
) -> jint {
    debug_assert!(!arr_cls.is_null());
    return crate::object::class::JClass::ref_size(JClassPtr::from_raw(arr_cls.as_raw() as _))
        as jint;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_sun_misc_Unsafe_addressSize<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
) -> jint {
    return crate::memory::POINTER_SIZE as jint;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_sun_misc_Unsafe_compareAndSwapObject<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
    o: JObject<'local>,
    offset: jlong,
    expected: JObject<'local>,
    x: JObject<'local>,
) -> jboolean {
    let target = ObjectPtr::from_raw(o.as_raw() as _);
    let val_ptr: Ptr<ObjectPtr> = target.read_value_ptr(offset as isize);

    unsafe {
        if let Ok(_) = AtomicPtr::from_ptr(val_ptr.as_mut_raw_ptr() as _).compare_exchange(
            expected.as_raw(),
            x.as_raw(),
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            return 1;
        }
    }
    return 0;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_sun_misc_Unsafe_compareAndSwapInt<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
    o: JObject<'local>,
    offset: jlong,
    expected: jint,
    x: jint,
) -> jboolean {
    let target = ObjectPtr::from_raw(o.as_raw() as _);
    let val_ptr: Ptr<JInt> = target.read_value_ptr(offset as isize);
    unsafe {
        if let Ok(_) = AtomicI32::from_ptr(val_ptr.as_mut_raw_ptr()).compare_exchange(
            expected,
            x,
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            return 1;
        }
    }
    return 0;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_sun_misc_Unsafe_putOrderedObject<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
    o: JObject<'local>,
    offset: jlong,
    x: JObject<'local>,
) {
    let target = ObjectPtr::from_raw(o.as_raw() as _);
    let val_ptr: Ptr<ObjectPtr> = target.read_value_ptr(offset as isize);

    unsafe {
        AtomicPtr::from_ptr(val_ptr.as_mut_raw_ptr() as _).store(x.as_raw(), Ordering::Relaxed);
    }
}
