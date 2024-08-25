use jni::{
    objects::{JClass, JObject, JString as JNIString},
    sys::{jint, jstring},
    JNIEnv,
};

use crate::{
    native::{
        java_io_FileSystem::{FS_BA_DIRECTORY, FS_BA_EXISTS},
        jni::JNIEnvWrapper,
    },
    object::string::{JString, JStringPtr},
    thread::Thread,
    ObjectPtr,
};

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_io_WinNTFileSystem_initIDs<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
) {
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_io_WinNTFileSystem_getBooleanAttributes<'local>(
    env: JNIEnv<'local>,
    _obj_ref: JClass<'local>,
    file: JObject<'local>,
) -> jint {
    let vm = JNIEnvWrapper::from_raw_env(env.get_raw()).vm();
    if file.is_null() {
        return 0;
    }
    let file = ObjectPtr::from_raw(file.as_raw() as _);
    let path = vm
        .shared_objs()
        .class_infos()
        .java_io_file_info()
        .get_path(file);
    let path = JString::to_rust_string(path, vm.as_ref());
    log::info!(
        "Java_java_io_WinNTFileSystem_getBooleanAttributes path {}",
        path
    );
    match std::fs::metadata(path) {
        Ok(metadata) => {
            let mut attrs = FS_BA_EXISTS;
            if metadata.is_dir() {
                attrs |= FS_BA_DIRECTORY;
            }
            return attrs;
        }
        Err(_e) => return 0,
    };
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_io_WinNTFileSystem_canonicalize0<'local>(
    env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
    path: JNIString<'local>,
) -> jstring {
    let vm = JNIEnvWrapper::from_raw_env(env.get_raw()).vm();
    let path = JStringPtr::from_raw(path.as_raw() as _);
    let rs_path = JString::to_rust_string(path, vm.as_ref());
    if let Ok(canon_path) = std::path::Path::new(&rs_path).canonicalize() {
        let canon_path = canon_path.to_str().unwrap();
        if canon_path == rs_path {
            return path.as_raw_ptr() as _;
        }
        let canon_path = JString::str_to_utf16(canon_path);
        let thread = Thread::current();
        let canon_path = vm
            .shared_objs()
            .class_infos()
            .java_lang_string_info()
            .create_with_utf16(&canon_path, thread);
        return canon_path.get_ptr().as_raw_ptr() as _;
    } else {
        todo!("throw IOException");
    }
}
