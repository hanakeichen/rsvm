use jni::{objects::JClass, sys::jlong, JNIEnv};

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_sun_io_Win32ErrorMode_setErrorMode<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
    m: jlong,
) -> jlong {
    #[cfg(target_os = "windows")]
    unsafe {
        use winapi::um::errhandlingapi::SetErrorMode;

        return SetErrorMode(m as _) as _;
    }
    #[cfg(not(target_os = "windows"))]
    unreachable!();
}
