use jni::{
    objects::{JClass, JObject},
    sys::{jclass, jint, jlong, jobject},
    JNIEnv,
};

use crate::{
    handle::Handle,
    object::{array::JArrayPtr, Object},
    thread::Thread,
    JArray, ObjectPtr,
};

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Object_registerNatives<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
) {
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Object_getClass<'local>(
    _env: JNIEnv<'local>,
    obj_ref: JObject<'local>,
) -> jclass {
    if obj_ref.is_null() {
        todo!("throw NullPointerException");
    }
    let obj_ref = ObjectPtr::from_raw(obj_ref.as_raw() as _);
    return obj_ref.jclass().as_raw_ptr() as _;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Object_hashCode<'local>(
    _env: JNIEnv<'local>,
    obj_ref: JObject<'local>,
) -> jint {
    type InternalObject = crate::object::Object;
    type InternalObjectPtr = crate::ObjectPtr;

    return InternalObjectPtr::from_raw(obj_ref.as_raw() as *mut InternalObject).hash();
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Object_clone<'local>(
    _env: JNIEnv<'local>,
    obj_ref: JObject<'local>,
) -> jobject {
    if obj_ref.is_null() {
        todo!("throw NullPointerException");
    }
    let obj_ref = ObjectPtr::from_raw(obj_ref.as_raw() as _);
    let obj_jcls = obj_ref.jclass();
    let thread = Thread::current();
    if obj_jcls.class_data().is_array() {
        let obj_ref: JArrayPtr = obj_ref.cast();
        let length = obj_ref.length();
        let result = Handle::new(JArray::new(length, obj_jcls, thread)).as_ptr();
        JArray::copy_unchecked(obj_ref, 0, result, 0, length);
        return result.as_raw_ptr() as _;
    }
    if !obj_jcls.is_implement(thread.vm().shared_objs().java_lang_cloneable_cls) {
        log::trace!("obj_ref jclass {}", obj_ref.jclass().name().as_str());
        todo!("throw CloneNotSupportedException");
    }
    return Object::clone(obj_ref, thread).as_ptr().as_raw_ptr() as _;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Object_notify<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
) {
    todo!();
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Object_notifyAll<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
) {
    // TODO
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Object_wait<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
    _timeout: jlong,
) {
    todo!();
}
