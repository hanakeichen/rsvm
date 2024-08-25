use jni::{objects::JClass, sys::{jint, jobject}, JNIEnv};

use crate::{handle::Handle, object::Object, thread::Thread};

use super::jni::JNIEnvWrapper;

/// see java.lang.FileSystem#BA_EXISTS
pub(super) const FS_BA_EXISTS: jint = 0x01;
/// see java.lang.FileSystem#BA_REGULAR
#[allow(unused)]
pub(super) const FS_BA_REGULAR: jint = 0x02;
/// see java.lang.FileSystem#BA_DIRECTORY
pub(super) const FS_BA_DIRECTORY: jint = 0x04;
/// see java.lang.FileSystem#BA_HIDDEN
#[allow(unused)]
pub(super) const FS_BA_HIDDEN: jint = 0x08;

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_io_FileSystem_getFileSystem<'local>(
    env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
) -> jobject {
    let vm = JNIEnvWrapper::from_raw_env(env.get_raw()).vm();
    let thread = Thread::current();
    let symbols = vm.shared_objs().symbols();
    let fs_cls_name = if cfg!(unix) {
        symbols.java_io_UnixFileSystem
    } else {
        symbols.java_io_WinNTFileSystem
    };
    match vm
        .bootstrap_class_loader
        .load_class_with_symbol(fs_cls_name)
    {
        Ok(unix_fs_cls) => {
            if let Err(_e) = unix_fs_cls.initialize(thread) {
                todo!();
            }
            match unix_fs_cls.resolve_self_method(symbols.ctor_init, symbols.noargs_retv_descriptor)
            {
                Ok(ctor) => {
                    let result = Handle::new(Object::new(unix_fs_cls, thread)).as_ptr();
                    vm.call_obj_void(result, ctor.method, &[]);
                    return result.as_raw_ptr() as _;
                }
                Err(_e) => todo!(),
            }
        }
        Err(_e) => todo!(),
    };
}
