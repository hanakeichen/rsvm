use jni::{objects::JClass, JNIEnv};

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_io_FileInputStream_initIDs<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
) {
}
