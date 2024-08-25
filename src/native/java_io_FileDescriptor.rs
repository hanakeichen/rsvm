use jni::{
    objects::JClass,
    sys::{jint, jlong},
    JNIEnv,
};

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_io_FileDescriptor_initIDs<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
) {
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_io_FileDescriptor_set<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
    d: jint,
) -> jlong {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::io::AsRawHandle;

        if d == 0 {
            return std::io::stdin().as_raw_handle() as _;
        } else if d == 1 {
            return std::io::stdout().as_raw_handle() as _;
        } else if d == 2 {
            return std::io::stderr().as_raw_handle() as _;
        }
    }
    return -1;
}
