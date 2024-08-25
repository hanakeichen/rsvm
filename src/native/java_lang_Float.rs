use std::mem::transmute;

use jni::{
    objects::JClass,
    sys::{jfloat, jint},
    JNIEnv,
};

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Float_floatToRawIntBits<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
    value: jfloat,
) -> jint {
    return unsafe { transmute(value) };
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Float_intBitsToFloat<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
    _bits: jint,
) -> jfloat {
    todo!();
}
