use std::{path::PathBuf, time::SystemTime};

use jni::{
    objects::{JClass, JObject, JString as JNIString},
    sys::{jint, jlong, jobject, jstring},
    JNIEnv,
};

use crate::{
    classfile::class_info::JavaUtilPropertiesInfo,
    object::{
        array::{self as vm_a, JArrayPtr},
        class::JClass as InternalJClass,
        string::{JString, JStringPtr, Utf16String},
    },
    thread::ThreadPtr,
    utils,
    vm::VMPtr,
    JClassPtr,
};

use crate::{thread::Thread, ObjectPtr};

use super::jni::JNIEnvWrapper;

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_System_registerNatives<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
) {
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_System_setIn0<'local>(
    env: JNIEnv<'local>,
    cls_ref: JClass<'local>,
    in_stream: JObject<'local>,
) {
    let vm = JNIEnvWrapper::from_raw_env(env.get_raw()).vm();
    let cls_ref = JClassPtr::from_raw(cls_ref.as_raw() as _);
    let in_stream = ObjectPtr::from_raw(in_stream.as_raw() as _);
    let (in_field, _) = cls_ref.get_field_with_name(vm.shared_objs().symbols().fd_in);
    in_field.set_static_value(cls_ref, in_stream);
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_System_setOut0<'local>(
    env: JNIEnv<'local>,
    cls_ref: JClass<'local>,
    out_stream: JObject<'local>,
) {
    let vm = JNIEnvWrapper::from_raw_env(env.get_raw()).vm();
    let cls_ref = JClassPtr::from_raw(cls_ref.as_raw() as _);
    let out_stream = ObjectPtr::from_raw(out_stream.as_raw() as _);
    let (out_field, _) = cls_ref.get_field_with_name(vm.shared_objs().symbols().fd_out);
    out_field.set_static_value(cls_ref, out_stream);
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_System_setErr0<'local>(
    env: JNIEnv<'local>,
    cls_ref: JClass<'local>,
    err_stream: JObject<'local>,
) {
    let vm = JNIEnvWrapper::from_raw_env(env.get_raw()).vm();
    let cls_ref = JClassPtr::from_raw(cls_ref.as_raw() as _);
    let err_stream = ObjectPtr::from_raw(err_stream.as_raw() as _);
    let (err_field, _) = cls_ref.get_field_with_name(vm.shared_objs().symbols().fd_err);
    err_field.set_static_value(cls_ref, err_stream);
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_System_currentTimeMillis<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
) -> jlong {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as _
}
#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_System_nanoTime<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
) -> jlong {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as _
}
#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_System_arraycopy<'local>(
    env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
    src: JObject<'local>,
    src_pos: jint,
    dest: JObject<'local>,
    dest_pos: jint,
    length: jint,
) {
    let src = JArrayPtr::from_raw(src.as_raw() as _);
    if src.is_null() {
        todo!("throw NullPointerException");
    }
    let dest = JArrayPtr::from_raw(dest.as_raw() as _);
    if dest.is_null() {
        todo!("throw NullPointerException");
    }
    if src_pos < 0 || dest_pos < 0 || length < 0 {
        todo!("throw IndexOutOfBoundsException");
    }
    let src_cls = src.jclass();
    let src_cls_data = src_cls.class_data();
    if !src_cls_data.is_array() {
        todo!("throw ArrayStoreException");
    }
    if src_pos + length > src.length() {
        todo!("throw IndexOutOfBoundsException");
    }
    let dest_cls_data = dest.jclass().class_data();
    if !dest_cls_data.is_array() {
        todo!("throw ArrayStoreException");
    }
    if dest_pos + length > dest.length() {
        todo!("throw IndexOutOfBoundsException");
    }
    let src_cmpt_cls = src_cls_data.component_type();
    let dest_cmpt_cls = dest_cls_data.component_type();
    let vm = JNIEnvWrapper::from_raw_env(env.get_raw()).vm();
    if !dest_cmpt_cls.is_assignable_from(src_cmpt_cls, vm) {
        todo!("throw ArrayStoreException");
    }

    log::trace!(
        "Java_java_lang_System_arraycopy src cls 0x{:x}",
        src_cmpt_cls.as_isize()
    );

    if InternalJClass::is_primitive(src_cmpt_cls) {
        if InternalJClass::is_byte_arr(src_cls, vm) {
            vm_a::JByteArray::copy_unchecked(src.cast(), src_pos, dest.cast(), dest_pos, length);
        } else if InternalJClass::is_char_arr(src_cls, vm) {
            vm_a::JCharArray::copy_unchecked(src.cast(), src_pos, dest.cast(), dest_pos, length);
        } else if InternalJClass::is_int_arr(src_cls, vm) {
            vm_a::JIntArray::copy_unchecked(src.cast(), src_pos, dest.cast(), dest_pos, length);
        } else if InternalJClass::is_long_arr(src_cls, vm) {
            vm_a::JLongArray::copy_unchecked(src.cast(), src_pos, dest.cast(), dest_pos, length);
        } else if InternalJClass::is_float_arr(src_cls, vm) {
            vm_a::JFloatArray::copy_unchecked(src.cast(), src_pos, dest.cast(), dest_pos, length);
        } else if InternalJClass::is_double_arr(src_cls, vm) {
            vm_a::JDoubleArray::copy_unchecked(src.cast(), src_pos, dest.cast(), dest_pos, length);
        } else if InternalJClass::is_short_arr(src_cls, vm) {
            vm_a::JShortArray::copy_unchecked(src.cast(), src_pos, dest.cast(), dest_pos, length);
        } else if InternalJClass::is_boolean_arr(src_cls, vm) {
            vm_a::JByteArray::copy_unchecked(src.cast(), src_pos, dest.cast(), dest_pos, length);
        } else {
            unreachable!();
        }
    } else {
        vm_a::JArray::copy_unchecked(src.cast(), src_pos, dest.cast(), dest_pos, length);
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_System_identityHashCode<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
    x: JObject<'local>,
) -> jint {
    let hash = ObjectPtr::from_raw(x.as_raw() as _).hash();
    debug_assert_ne!(hash, 0);
    hash
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_System_initProperties<'local>(
    env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
    jni_props: JObject<'local>,
) -> jobject {
    let vm = JNIEnvWrapper::from_raw_env(env.get_raw()).vm();
    let thread = Thread::current();
    let props = ObjectPtr::from_raw(jni_props.as_raw() as _);
    let props_cls_info = vm.shared_objs().class_infos().java_util_properties_info();
    sys_put_file_encoding(props, props_cls_info, vm, thread);
    sys_put_file_separator(props, props_cls_info, vm, thread);
    sys_put_path_separator(props, props_cls_info, vm, thread);
    sys_put_line_separator(props, props_cls_info, vm, thread);
    sys_put_boot_lib_path(props, props_cls_info, vm, thread);
    sys_put_java_home(props, props_cls_info, vm, thread);
    return jni_props.as_raw();
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_System_mapLibraryName<'local>(
    env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
    libname: JNIString<'local>,
) -> jstring {
    let vm = JNIEnvWrapper::from_raw_env(env.get_raw()).vm();
    let libname = JStringPtr::from_raw(libname.as_raw() as _);
    let mut rs_libname = JString::to_rust_string(libname, vm.as_ref());
    if cfg!(target_os = "linux") {
        rs_libname.insert_str(0, "lib");
        rs_libname.push_str(".so");
    } else if cfg!(target_os = "macos") {
        rs_libname.insert_str(0, "lib");
        rs_libname.push_str(".dylib");
    } else if cfg!(windows) {
        rs_libname.push_str(".dll");
    } else {
        todo!();
    };
    let rs_libname = JString::str_to_utf16(&rs_libname);
    let result = vm
        .shared_objs()
        .class_infos()
        .java_lang_string_info()
        .create_with_utf16(&rs_libname, Thread::current());
    return result.get_ptr().as_raw_ptr() as _;
}

fn sys_put_file_encoding(
    props: ObjectPtr,
    props_cls_info: &JavaUtilPropertiesInfo,
    vm: VMPtr,
    thread: ThreadPtr,
) {
    let k_encoding = vm.get_jstr_from_symbol(vm.get_symbol("file.encoding"), thread);
    let v_utf8 = vm.get_jstr_from_symbol(vm.get_symbol("UTF-8"), thread);
    props_cls_info.put(props, k_encoding.cast(), v_utf8.cast(), vm);
}

fn sys_put_file_separator(
    props: ObjectPtr,
    props_cls_info: &JavaUtilPropertiesInfo,
    vm: VMPtr,
    thread: ThreadPtr,
) {
    let k_file_separator = vm.get_jstr_from_symbol(vm.get_symbol("file.separator"), thread);
    let v_file_separator = if cfg!(unix) {
        "/"
    } else if cfg!(windows) {
        "\\"
    } else {
        todo!();
    };
    let v_file_separator = vm.get_jstr_from_symbol(vm.get_symbol(v_file_separator), thread);
    props_cls_info.put(props, k_file_separator.cast(), v_file_separator.cast(), vm);
}

fn sys_put_path_separator(
    props: ObjectPtr,
    props_cls_info: &JavaUtilPropertiesInfo,
    vm: VMPtr,
    thread: ThreadPtr,
) {
    let k_path_separator = vm.get_jstr_from_symbol(vm.get_symbol("path.separator"), thread);
    let v_path_separator = utils::get_path_separator();
    let v_file_separator = vm.get_intern_jstr(&JString::str_to_utf16(&v_path_separator), thread);
    props_cls_info.put(props, k_path_separator.cast(), v_file_separator.cast(), vm);
}

fn sys_put_line_separator(
    props: ObjectPtr,
    props_cls_info: &JavaUtilPropertiesInfo,
    vm: VMPtr,
    thread: ThreadPtr,
) {
    let k_line_separator = vm.get_jstr_from_symbol(vm.get_symbol("line.separator"), thread);
    let v_line_separator = if cfg!(target_os = "linux") {
        "\n"
    } else if cfg!(target_os = "macos") {
        "\n"
    } else if cfg!(windows) {
        "\r\n"
    } else {
        todo!();
    };
    let v_line_separator = vm.get_intern_jstr(&JString::str_to_utf16(&v_line_separator), thread);
    props_cls_info.put(props, k_line_separator.cast(), v_line_separator.cast(), vm);
}

fn sys_put_boot_lib_path(
    props: ObjectPtr,
    props_cls_info: &JavaUtilPropertiesInfo,
    vm: VMPtr,
    thread: ThreadPtr,
) {
    let k_boot_lib_path =
        vm.get_intern_jstr(&JString::str_to_utf16("sun.boot.library.path"), thread);

    let v_boot_lib_path: Utf16String = if let Some(boot_lib_path) = vm.cfg.boot_lib_path() {
        JString::str_to_utf16(boot_lib_path)
    } else {
        let mut boot_lib_path = PathBuf::new();
        boot_lib_path.push(vm.cfg.rsvm_home());
        boot_lib_path.push("lib");
        let boot_lib_path = boot_lib_path.to_str().unwrap();
        JString::str_to_utf16(boot_lib_path)
    };
    let v_boot_lib_path = vm
        .shared_objs()
        .class_infos()
        .java_lang_string_info()
        .create_permanent_with_utf16(&v_boot_lib_path, thread);
    props_cls_info.put(
        props,
        k_boot_lib_path.cast(),
        v_boot_lib_path.get_ptr().cast(),
        vm,
    );
}

fn sys_put_java_home(
    props: ObjectPtr,
    props_cls_info: &JavaUtilPropertiesInfo,
    vm: VMPtr,
    thread: ThreadPtr,
) {
    let v_java_home = JString::str_to_utf16(vm.cfg.rsvm_home());

    let k_java_home = vm.get_intern_jstr(&JString::str_to_utf16("java.home"), thread);
    let v_java_home = vm
        .shared_objs()
        .class_infos()
        .java_lang_string_info()
        .create_permanent_with_utf16(&v_java_home, thread);
    props_cls_info.put(props, k_java_home.cast(), v_java_home.get_ptr().cast(), vm);
}
