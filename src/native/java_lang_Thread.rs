use jni::{
    objects::JClass,
    sys::{jint, jobject},
    JNIEnv,
};

use crate::thread::Thread;

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Thread_registerNatives<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
) {
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Thread_currentThread<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
) -> jobject {
    return Thread::current().jthread().as_raw_ptr() as _;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Thread_setPriority0<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JClass<'local>,
    _new_priority: jint,
) {
    // TODO
}
