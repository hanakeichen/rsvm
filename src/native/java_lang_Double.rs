use std::mem::transmute;

use jni::{
    objects::JClass,
    sys::{jdouble, jlong},
    JNIEnv,
};

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Double_doubleToRawLongBits<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
    value: jdouble,
) -> jlong {
    return unsafe { transmute(value) };
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Double_longBitsToDouble<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
    _bits: jlong,
) -> jdouble {
    todo!();
}
