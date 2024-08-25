use jni::{objects::JObject, sys::jstring, JNIEnv};

use crate::{object::string::JStringPtr, thread::Thread};

use super::jni::JNIEnvWrapper;

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_String_intern<'local>(
    env: JNIEnv<'local>,
    obj_ref: JObject<'local>,
) -> jstring {
    let vm = JNIEnvWrapper::from_raw_env(env.get_raw()).vm();
    let result = vm.intern_jstr(
        JStringPtr::from_raw(obj_ref.as_raw() as _),
        Thread::current(),
    );
    return result.as_raw_ptr() as jstring;
}
