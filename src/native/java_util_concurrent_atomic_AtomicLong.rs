use jni::{objects::JClass, sys::jboolean, JNIEnv};

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_util_concurrent_atomic_AtomicLong_VMSupportsCS8<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
) -> jboolean {
    return 1;
}
