use jni::{
    objects::JClass,
    sys::{jclass, jint},
    JNIEnv,
};

use crate::{thread::Thread, JClassPtr};

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_sun_reflect_Reflection_getCallerClass<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
) -> jclass {
    let caller_cls = Thread::current().interpreter().grand_parent_stack_class();
    log::trace!(
        "Java_sun_reflect_Reflection_getCallerClass {}",
        caller_cls.name().as_str()
    );
    caller_cls.as_mut_raw_ptr() as jclass
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_sun_reflect_Reflection_getClassAccessFlags<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
    c: JClass<'local>,
) -> jint {
    return JClassPtr::from_raw(c.as_raw() as _)
        .class_data()
        .access_flags() as jint;
}
