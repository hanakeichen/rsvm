use std::ptr::null_mut;

use jni::sys::JNINativeInterface_;

use crate::{object::prelude::Ptr, vm::VMPtr};

pub(crate) type JNIEnvWrapperPtr = Ptr<JNIEnvWrapper>;

pub(crate) struct JNIEnvWrapper {
    #[allow(unused)]
    env: jni::sys::JNIEnv,
    vm: VMPtr,
}

impl JNIEnvWrapper {
    fn default() -> Self {
        Self {
            env: null_mut(),
            vm: VMPtr::null(),
        }
    }

    pub fn from_raw_env(env: *mut jni::sys::JNIEnv) -> JNIEnvWrapperPtr {
        JNIEnvWrapperPtr::from_raw(env as *mut JNIEnvWrapper)
    }

    pub fn vm(&self) -> VMPtr {
        self.vm
    }
}

pub(crate) struct JNIWrapper {
    #[allow(unused)]
    jni: JNINativeInterface_,
    env_wrapper: JNIEnvWrapper,
}

impl JNIWrapper {
    pub fn default() -> Self {
        Self {
            jni: unsafe { std::mem::zeroed() },
            env_wrapper: JNIEnvWrapper::default(),
        }
    }

    pub fn init(&mut self, vm: VMPtr) {
        self.env_wrapper.env = &self.jni;
        self.env_wrapper.vm = vm;
    }

    pub fn get_env_handle(&self) -> isize {
        unsafe {
            std::mem::transmute(&self.env_wrapper as *const JNIEnvWrapper as *mut jni::sys::JNIEnv)
        }
    }
}
