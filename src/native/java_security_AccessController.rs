use std::ptr::null_mut;

use jni::{
    objects::{JClass, JObject},
    sys::jobject,
    JNIEnv,
};

use crate::ObjectPtr;

use super::jni::JNIEnvWrapper;

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_security_AccessController_doPrivileged<'local>(
    env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
    action: JObject<'local>,
) -> jobject {
    type InternalJClass = crate::object::class::JClass;

    let vm = JNIEnvWrapper::from_raw_env(env.get_raw()).vm();
    let action = ObjectPtr::from_raw(action.as_raw() as _);
    let action_cls_info = vm
        .shared_objs()
        .class_infos()
        .java_security_privileged_action_info();
    let action_cls = action_cls_info.cls();
    let run_method = match InternalJClass::resolve_interface_method(
        action.jclass(),
        action_cls,
        action_cls_info.run_name(),
        action_cls_info.run_descriptor(),
    ) {
        Ok(resolved) => resolved.method,
        Err(_e) => todo!(),
    };
    let result = vm.call_obj(action, run_method, &[]);
    return result.obj_val().as_raw_ptr() as _;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_security_AccessController_getStackAccessControlContext<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
) -> jobject {
    return null_mut();
}
