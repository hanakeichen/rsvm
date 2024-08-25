use crate::{
    classfile::class_info::JavaIOFileDescriptorInfo, object::array::JByteArrayPtr, ObjectPtr,
};
use jni::{
    objects::{JByteArray, JClass},
    sys::{jboolean, jint},
    JNIEnv,
};
use std::{
    fs::File,
    io::{Seek, SeekFrom, Write},
    mem::transmute,
};

use super::jni::JNIEnvWrapper;

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_io_FileOutputStream_initIDs<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
) {
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_io_FileOutputStream_writeBytes<'local>(
    env: JNIEnv<'local>,
    obj_ref: JClass<'local>,
    bytes: JByteArray<'local>,
    off: jint,
    len: jint,
    append: jboolean,
) {
    let vm = JNIEnvWrapper::from_raw_env(env.get_raw()).vm();
    let bytes = JByteArrayPtr::from_raw(bytes.as_raw() as _);
    let bytes_len = bytes.length();
    let end_idx = off + len;
    if end_idx >= bytes_len {
        todo!("throw IOException");
    }

    let obj_ref = ObjectPtr::from_raw(obj_ref.as_raw() as _);
    let cls_infos = vm.shared_objs().class_infos();
    let fd = cls_infos.java_io_file_output_stream_info().get_fd(obj_ref);
    let fd_cls_info = cls_infos.java_io_file_descriptor_info();
    let buf = bytes.data();
    let bytes = &buf.as_slice(bytes_len as usize)[off as usize..end_idx as usize];
    let bytes = unsafe { transmute(bytes) };
    let mut file = get_file_from_raw(fd_cls_info, fd);
    if append == 1 {
        if let Err(_e) = file.seek(SeekFrom::End(0)) {
            todo!("throw IOException");
        }
    }
    if let Err(_e) = file.write_all(bytes) {
        todo!("throw IOException");
    }
    std::mem::forget(file);
}

fn get_file_from_raw(fd_cls_info: &JavaIOFileDescriptorInfo, fd: ObjectPtr) -> File {
    #[cfg(target_family = "unix")]
    {
        use std::os::fd::FromRawFd;

        let fd = fd_cls_info.get_fd(fd);
        unsafe { File::from_raw_fd(fd) }
    }
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::io::FromRawHandle;

        let fd = fd_cls_info.get_handle(fd);
        unsafe { File::from_raw_handle(fd as _) }
    }
}
