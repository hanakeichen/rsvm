use jni::{
    objects::JObject,
    sys::{jint, jlong},
    JNIEnv,
};

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Runtime_availableProcessors<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
) -> jint {
    return std::thread::available_parallelism().unwrap().get() as _;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Runtime_freeMemory<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
) -> jlong {
    // TODO
    return 0;
}
