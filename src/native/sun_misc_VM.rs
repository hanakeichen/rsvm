use jni::{objects::JClass, JNIEnv};

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_sun_misc_VM_initialize<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
) {
}
