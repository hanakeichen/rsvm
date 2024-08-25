use jni::{objects::JClass, JNIEnv};

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_System_dummyZip<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
) {
}
