use crate::{
    handle::HandleScope,
    object::{class::JClassPtr, method::MethodPtr, prelude::Ptr},
    thread::ThreadPtr,
};

pub type FramePtr = Ptr<Frame>;

pub struct Frame {
    class: JClassPtr,
    method: MethodPtr,
    prev: FramePtr,
    frame_slots: isize,
    is_java_top: bool,
    _scope: HandleScope,
}

impl Frame {
    pub fn new(
        class: JClassPtr,
        method: MethodPtr,
        prev: FramePtr,
        frame_slots: isize,
        is_java_top: bool,
        thread: ThreadPtr,
    ) -> FramePtr {
        let scope = HandleScope::new(thread);
        return FramePtr::new(Box::into_raw(Box::new(Frame {
            class,
            method,
            prev,
            frame_slots,
            is_java_top,
            _scope: scope,
        })));
    }

    pub fn destroy(frame: FramePtr) {
        unsafe {
            let _ = Box::from_raw(frame.as_mut_raw_ptr());
        }
    }

    #[inline]
    pub fn class(&self) -> JClassPtr {
        self.class
    }

    #[inline]
    pub fn method(&self) -> MethodPtr {
        self.method
    }

    #[inline]
    pub fn prev(&self) -> FramePtr {
        self.prev
    }

    #[inline]
    pub fn has_prev(&self) -> bool {
        return self.prev.is_not_null();
    }

    #[inline]
    pub fn frame_slots(&self) -> isize {
        self.frame_slots
    }

    #[inline]
    pub fn is_java_top(&self) -> bool {
        self.is_java_top
    }
}
