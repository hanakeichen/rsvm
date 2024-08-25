use jni::{
    objects::{JClass, JObject},
    JNIEnv,
};
use libloading::{Library, Symbol};

use crate::{
    memory::Address,
    native::jni::JNIEnvWrapper,
    object::{
        class::ClassData,
        method::MethodPtr,
        prelude::{JLong, ObjectRawPtr, Ptr},
        string::{JString, JStringPtr},
    },
    ObjectPtr,
};

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_ClassLoader_registerNatives<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
) {
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_ClassLoader_NativeLibrary_load<'local>(
    env: JNIEnv<'local>,
    obj_ref: JObject<'local>,
    lib: JObject<'local>,
) {
    let vm = JNIEnvWrapper::from_raw_env(env.get_raw()).vm();
    let lib = JStringPtr::from_raw(lib.as_raw() as _);
    let lib = JString::to_rust_string(lib, vm.as_ref());

    let lib = unsafe {
        match Library::new(lib) {
            Ok(lib) => Ptr::new(Box::into_raw(Box::new(lib))),
            Err(e) => {
                log::info!("Java_java_lang_ClassLoader_NativeLibrary_load failed {:#?}", e);
                return;
            }
        }
    };

    let obj_ref = ObjectPtr::from_raw(obj_ref.as_raw() as _);
    let class_info = vm
        .shared_objs()
        .class_infos()
        .java_lang_classloader_native_library_info();
    class_info.set_handle(obj_ref, lib.as_isize() as JLong);
    let from_cls_name = class_info.get_from_class(obj_ref).name();

    let from_cls = class_info.get_from_class(obj_ref);
    let methods = from_cls.class_data().methods();
    for idx in 0..methods.length() {
        let mut method: MethodPtr = methods.get(idx).cast();
        if method.is_native() {
            let native_fn_name =
                ClassData::get_native_fn_name(from_cls_name.as_str(), method.name().as_str());
            unsafe {
                if let Ok(symbol) = lib.get(native_fn_name.as_bytes()) {
                    let symbol: Symbol<ObjectRawPtr> = symbol;
                    if let Some(native_fn) = symbol.try_as_raw_ptr() {
                        method.set_native_fn(Address::from_c_ptr(native_fn));
                    }
                } else {
                    continue;
                }
            }
        }
    }
}
