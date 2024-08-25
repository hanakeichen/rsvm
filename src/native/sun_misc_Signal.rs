use std::{ffi::c_void, os::raw::c_int};

use jni::{
    objects::{JClass, JObject},
    sys::{jint, jlong},
    JNIEnv,
};
use libc::sighandler_t;

use crate::{
    object::{
        method::MethodPtr,
        string::{JString, JStringPtr},
    },
    value::JValue,
    vm::VMPtr,
    JClassPtr,
};

use super::jni::JNIEnvWrapper;

static mut SIGNAL_CLS: JClassPtr = JClassPtr::null();
static mut VM: VMPtr = VMPtr::null();
static mut DISPATCH_METHOD: MethodPtr = MethodPtr::null();

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_sun_misc_Signal_findSignal<'local>(
    env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
    sig_name: JObject<'local>,
) -> jint {
    let vm = JNIEnvWrapper::from_raw_env(env.get_raw()).vm();
    let sig_name = JStringPtr::from_raw(sig_name.as_raw() as _);
    let sig_name = JString::to_rust_string(sig_name, vm.as_ref());

    log::trace!("Java_sun_misc_Signal_findSignal {}", sig_name);

    match sig_name.as_str() {
        #[cfg(target_family = "unix")]
        "HUP" => libc::SIGHUP,
        "INT" => libc::SIGINT,
        #[cfg(target_family = "unix")]
        "QUIT" => libc::SIGQUIT,
        "ILL" => libc::SIGILL,
        "ABRT" => libc::SIGABRT,
        #[cfg(target_os = "macos")]
        "EMT" => libc::SIGEMT,
        "FPE" => libc::SIGFPE,
        #[cfg(target_family = "unix")]
        "KILL" => libc::SIGKILL,
        "SEGV" => libc::SIGSEGV,
        #[cfg(target_family = "unix")]
        "PIPE" => libc::SIGPIPE,
        #[cfg(target_family = "unix")]
        "ALRM" => libc::SIGALRM,
        "TERM" => libc::SIGTERM,
        _ => -1,
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_sun_misc_Signal_handle0<'local>(
    env: JNIEnv<'local>,
    cls_ref: JClass<'local>,
    sig: jint,
    native_h: jlong,
) -> jlong {
    if native_h == 0 {
        // 0     default handler
        return unsafe { libc::signal(sig, libc::SIG_DFL) as jlong };
    }
    if native_h == 1 {
        // 1     ignore the signal
        return unsafe { libc::signal(sig, libc::SIG_IGN) as jlong };
    }
    if native_h == 2 {
        // 2     call back to Signal.dispatch
        let vm = JNIEnvWrapper::from_raw_env(env.get_raw()).vm();
        let cls_ref = JClassPtr::from_raw(cls_ref.as_raw() as _);
        let dispatch_method = cls_ref
            .resolve_local_method_unchecked(vm.get_symbol("dispatch"), vm.get_symbol("(I)V"));
        unsafe {
            SIGNAL_CLS = cls_ref;
            VM = vm;
            DISPATCH_METHOD = dispatch_method;
        }
        return unsafe { libc::signal(sig, get_signal_handler()) as jlong };
    }
    return -1;
}

fn get_signal_handler() -> sighandler_t {
    sig_handler as extern "C" fn(c_int) as *mut c_void as sighandler_t
}

extern "C" fn sig_handler(number: c_int) {
    unsafe {
        VM.call_static_void(SIGNAL_CLS, DISPATCH_METHOD, &[JValue::with_int_val(number)]);
    }
}

unsafe impl Sync for JClassPtr {}
