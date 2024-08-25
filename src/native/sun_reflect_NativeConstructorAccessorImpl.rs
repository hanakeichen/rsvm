use jni::{sys::jobject, JNIEnv};

use crate::{
    handle::Handle,
    object::{array::JArrayPtr, class::JClass, Object},
    thread::Thread,
    value::JValue,
    JClassPtr, ObjectPtr,
};

use super::jni::JNIEnvWrapper;

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_sun_reflect_NativeConstructorAccessorImpl_newInstance0<'local>(
    env: JNIEnv<'local>,
    _cls_ref: jni::objects::JClass<'local>,
    ctor: jni::objects::JObject<'local>,
    args: jni::objects::JObject<'local>,
) -> jobject {
    let vm = JNIEnvWrapper::from_raw_env(env.get_raw()).vm();
    let class_infos = vm.shared_objs().class_infos();
    let ctor_info = class_infos.java_lang_reflect_constructor_info();

    let ctor = ObjectPtr::from_raw(ctor.as_raw() as _);
    let decl_cls = ctor_info.get_decl_cls(ctor);
    let slot = ctor_info.get_slot(ctor);
    let ctor_init = decl_cls.get_method_with_index(slot);
    if ctor_init.is_null() {
        todo!("throw InvocationTargetException");
    }
    let args = JArrayPtr::from_raw(args.as_raw() as _);
    let args_len = if args.is_not_null() { args.length() } else { 0 };
    let native_params = ctor_init.params();
    if args_len != native_params.length() {
        todo!("throw IllegalArgumentException");
    }
    let mut j_args = Vec::with_capacity(args_len as usize);
    for idx in 0..args_len {
        let param_type: JClassPtr = native_params.get(idx).cast();
        let arg = args.get(idx);
        if arg.is_null() {
            if JClass::is_primitive(param_type) {
                todo!("throw IllegalArgumentException");
            }
            j_args.push(JValue::with_obj_null());
            continue;
        }
        let arg_cls = arg.jclass();
        if param_type.is_assignable_from(arg_cls, vm) {
            j_args.push(JValue::with_obj_val(arg));
            continue;
        } else if JClass::is_primitive(param_type) {
            if let Some(val) = class_infos.get_unboxed_jnumber(arg) {
                j_args.push(val);
                continue;
            }
        }
        todo!("throw IllegalArgumentException");
    }
    debug_assert_eq!(j_args.len(), native_params.length() as usize);
    let thread = Thread::current();
    let result = Handle::new(Object::new(decl_cls, thread)).as_ptr();
    vm.call_obj_void(result, ctor_init, &j_args);
    return result.as_raw_ptr() as _;
}
