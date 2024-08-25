use crate::{
    memory::{is_align_of, Address, POINTER_SIZE},
    object::{
        class::JClassPtr,
        method::MethodPtr,
        prelude::{JDouble, JFloat, JInt, JLong, ObjectPtr, ObjectRawPtr},
        Object,
    },
    thread::{Thread, ThreadPtr},
};

use super::frame::{Frame, FramePtr};

type StackSlot = *mut std::ffi::c_void;
type StackAddress = *mut StackSlot;

pub struct Stack {
    stack_base: StackAddress,
    stack_limit: StackAddress,
    sp: StackAddress,
    bp: StackAddress,
    frame: FramePtr,
    time: std::time::SystemTime,
}

impl Stack {
    pub fn new(stack_addr: Address, stack_size: usize) -> Stack {
        let stack_base = stack_addr.uoffset(stack_size).raw_ptr() as StackAddress;
        let stack_limit = stack_addr.raw_ptr() as StackAddress;

        return Stack {
            stack_base,
            stack_limit,
            sp: stack_base,
            bp: stack_base,
            frame: FramePtr::null(),
            time: std::time::SystemTime::now(),
        };
    }

    #[inline(always)]
    pub fn new_call_frame(
        &mut self,
        class: JClassPtr,
        method: MethodPtr,
        args_slots: isize,
        obj_ref_size: isize,
        pc: Address,
        is_java_top: bool,
        thread: ThreadPtr,
    ) {
        self.time = std::time::SystemTime::now();
        debug_assert!(
            method.max_locals() as isize >= args_slots,
            "trace {}#{}",
            class.name().as_str(),
            method.name().as_str()
        );
        debug_assert!(
            args_slots >= method.params().length() as isize + obj_ref_size,
            "trace {}#{}",
            class.name().as_str(),
            method.name().as_str()
        );
        let max_locals = method.max_locals() as isize;
        let prev_sp = unsafe { self.sp.offset(args_slots) };
        let prev_bp = self.bp;
        self.frame = Frame::new(
            method.decl_cls(),
            method,
            self.frame,
            max_locals,
            is_java_top,
            thread,
        );
        self.bp = prev_sp;
        self.sp = unsafe { self.bp.offset(-(max_locals + 3)) };
        log::trace!(
            "saved prev_sp {:?} prev_bp {:?} pc {:?}, current sp {:?}, bp {:?}, call {}:{}, desc {}, max_locals {}, args_slots {}, frame_slots {}",
            prev_sp,
            prev_bp,
            pc,
            self.sp,
            self.bp,
            class.name().as_str(),
            method.name().as_str(),
            method.descriptor().as_str(),
            method.max_locals(),
            args_slots,
            max_locals + 3,
        );
        if obj_ref_size == 1 {
            let obj_ref = self.load_jobj(0);
            log::trace!("new_call_frame objref: 0x{:x}", obj_ref.as_isize());
            debug_assert!(
                obj_ref.is_not_null()
                    && class.is_assignable_from(obj_ref.jclass(), thread.vm_ptr()),
                "{}",
                self.stack_trace_str()
            );
        }
        unsafe {
            self.store_jobj(std::mem::transmute(prev_sp), max_locals);
            self.store_jobj(std::mem::transmute(prev_bp), max_locals + 1);
            self.store_jobj(std::mem::transmute(pc), max_locals + 2);
        }
    }

    pub fn new_native_call_frame(
        &mut self,
        class: JClassPtr,
        method: MethodPtr,
        args_slots: isize,
        obj_ref_size: isize,
        pc: Address,
        is_java_top: bool,
        thread: ThreadPtr,
    ) {
        self.time = std::time::SystemTime::now();
        let prev_sp = unsafe { self.sp.offset(args_slots) };
        let prev_bp = self.bp;
        self.frame = Frame::new(
            method.decl_cls(),
            method,
            self.frame,
            args_slots,
            is_java_top,
            thread,
        );
        self.bp = prev_sp;
        self.sp = unsafe { self.bp.offset(-(args_slots as isize + 3)) };
        log::trace!(
                "saved prev_sp {:?} prev_bp {:?} pc {:?}, current sp {:?}, bp {:?}, call {}:{}, locals {}, {}",
                prev_sp,
                prev_bp,
                pc,
                self.sp,
                self.bp,
                class.name().as_str(),
                method.name().as_str(),
                method.max_locals(),
                args_slots,
            );
        if obj_ref_size == 1 {
            let obj_ref = self.load_jobj(0);
            log::trace!("new_call_frame objref: 0x{:x}", obj_ref.as_isize());
            debug_assert!(obj_ref.is_not_null(), "{}", self.stack_trace_str());
        }
        unsafe {
            self.store_jobj(std::mem::transmute(prev_sp), args_slots);
            self.store_jobj(std::mem::transmute(prev_bp), args_slots + 1);
            self.store_jobj(std::mem::transmute(pc), args_slots + 2);
        }
    }

    #[inline(always)]
    pub fn ret_call_frame(&mut self, set_pc: &mut Address) {
        {
            let elapsed = self.time.elapsed().unwrap().as_millis();
            if elapsed > 100 {
                log::info!(
                    "call {}#{} cost {}",
                    self.frame.class().name().as_str(),
                    self.frame.method().name().as_str(),
                    elapsed
                );
            }
        }
        let frame_locals = self.frame.frame_slots();
        let prev_sp = self.load_jobj_raw(frame_locals);
        let prev_bp = self.load_jobj_raw(frame_locals + 1);
        let prev_pc = self.load_jobj_raw(frame_locals + 2);
        log::trace!("restore {:x?} {:x?} {:x?}", prev_sp, prev_bp, prev_pc);
        unsafe {
            self.sp = std::mem::transmute(prev_sp);
            self.bp = std::mem::transmute(prev_bp);
            *set_pc = std::mem::transmute(prev_pc);
        }
        {
            let frame = self.frame;
            self.frame = frame.prev();
            Frame::destroy(frame);
        }
        if self.frame.is_not_null()
            && !self.frame.method().is_static()
            && self.frame.method().name().as_str() != "<clinit>"
        {
            log::trace!(
                "check obj_ref, class addr 0x{:x}, obj_ref jclass addr 0x{:x}, method {}",
                self.frame.class().as_isize(),
                self.load_jobj(0).jclass().as_isize(),
                self.frame.method().name().as_str()
            );

            debug_assert!(
                self.frame
                    .class()
                    .is_assignable_from(self.load_jobj(0).jclass(), Thread::current().vm_ptr()),
                "{}",
                self.stack_trace_str()
            );
        }
    }

    #[inline(always)]
    pub fn pop_jobj(&mut self) -> ObjectPtr {
        debug_assert!(self.sp.addr() < self.bp.addr());
        let val;
        unsafe {
            val = ObjectPtr::from_c_ptr(*self.sp);
            self.sp = self.sp.offset(1);
        }
        return val;
    }

    #[inline(always)]
    pub fn peek_int(&self, index: isize) -> JInt {
        unsafe { *(self.sp.offset(index) as *mut JInt) }
    }

    #[inline(always)]
    pub fn peek_jobj(&self) -> ObjectPtr {
        debug_assert!(self.sp.addr() < self.bp.addr());
        let val;
        unsafe {
            val = ObjectPtr::from_c_ptr(*self.sp);
        }
        return val;
    }

    #[inline(always)]
    pub fn peek_slot(&self) -> StackSlot {
        debug_assert!(self.sp.addr() < self.bp.addr());
        return unsafe { *self.sp };
    }

    #[inline(always)]
    pub fn pop<T: StackPrimitiveValue + Copy>(&mut self) -> T {
        debug_assert!(self.sp.addr() < self.bp.addr());
        let slots = Self::calc_slots::<T>();
        let val;
        unsafe {
            val = *(self.sp as *mut T);
            self.sp = self.sp.offset(slots);
        }
        return val;
    }

    #[inline(always)]
    pub fn pop_slot(&mut self) -> StackSlot {
        debug_assert!(self.sp.addr() < self.bp.addr());
        let val;
        unsafe {
            val = *self.sp;
            self.sp = self.sp.offset(1);
        }
        return val;
    }

    #[inline(always)]
    pub fn discard<T: StackPrimitiveValue + Copy>(&mut self) {
        debug_assert!(self.sp.addr() < self.bp.addr());
        let slots = Self::calc_slots::<T>();
        unsafe {
            self.sp = self.sp.offset(slots);
        }
    }

    #[inline(always)]
    pub fn push_jobj(&mut self, val: ObjectPtr) {
        debug_assert!(val.is_null() || val.jclass().name().is_not_null());
        log::trace!("push_jobj val 0x{:x}", val.as_isize());
        unsafe {
            debug_assert!(is_align_of(self.sp as usize, POINTER_SIZE));
            *self.sp.offset(-1) = val.as_c_ptr();
            self.sp = self.sp.offset(-1);
            debug_assert!(is_align_of(self.sp as usize, POINTER_SIZE));
            // *self.sp = val.as_isize();
        }
    }

    // TODO push char 的时候错误
    #[inline(always)]
    pub fn push<T: StackPrimitiveValue>(&mut self, val: T) {
        let slots = Self::calc_slots::<T>();
        log::trace!(
            "before push 0x{:x}, 0x{:x}, slots {}",
            self.sp.addr(),
            self.bp.addr(),
            slots
        );
        unsafe {
            self.sp = self.sp.offset(-slots);
            *(self.sp as *mut T) = val;
        }
        debug_assert!(is_align_of(self.sp as usize, 8));
        log::trace!(
            "after push 0x{:x}, 0x{:x}, slots {}",
            self.sp.addr(),
            self.bp.addr(),
            slots
        );
    }

    #[inline(always)]
    pub fn push_slot(&mut self, val: StackSlot) {
        unsafe {
            self.sp = self.sp.offset(-1);
            *self.sp = val;
        }
    }

    #[inline(always)]
    pub fn load_jobj(&self, index: isize) -> ObjectPtr {
        return ObjectPtr::from_raw(self.load_jobj_raw(index));
    }

    #[inline(always)]
    pub fn load_jobj_raw(&self, index: isize) -> ObjectRawPtr {
        debug_assert!(self.sp.addr() < self.bp.addr());
        let result = unsafe { *(self.bp.offset(-(index + 1)) as *const ObjectRawPtr) };
        log::trace!(
            "load_jobj==addr : {:x?}==={:x?}",
            unsafe { self.bp.offset(-(index + 1)) },
            result
        );
        return result;
    }

    #[inline(always)]
    pub fn load_callee_objref(&self, args_slots: isize) -> ObjectPtr {
        debug_assert!(self.sp.addr() < self.bp.addr());
        unsafe { ObjectPtr::from_raw(*(self.sp.offset(args_slots - 1) as *const ObjectRawPtr)) }
    }

    #[inline(always)]
    pub fn load<T>(&self, index: isize) -> T
    where
        T: StackPrimitiveValue + Copy,
    {
        debug_assert!(self.sp.addr() < self.bp.addr());
        let slots = Self::calc_slots::<T>();
        unsafe {
            log::trace!(
                "load 0x{:x}, 0x{:x} {:?}, index {}",
                self.sp.addr(),
                self.bp.addr(),
                self.bp.offset(-(index + slots)),
                index
            );
        }
        return unsafe { *(self.bp.offset(-(index + slots)) as *mut T) };
    }

    #[inline(always)]
    pub fn store<T: StackPrimitiveValue>(&self, val: T, index: isize) {
        let slots = Self::calc_slots::<T>();
        unsafe {
            (self.bp.offset(-(index + slots)) as *mut T).write(val);
        }
    }

    #[inline(always)]
    pub fn iinc(&self, const_val: JInt, index: isize) {
        unsafe {
            *(self.bp.offset(-(index + 1)) as *mut JInt) += const_val;
        }
    }

    #[inline(always)]
    pub fn swap(&self) {
        unsafe {
            let a_ptr = self.sp.offset(0) as *mut *mut Object;
            let b_ptr = self.sp.offset(1) as *mut *mut Object;

            let x = *a_ptr;
            let y = *b_ptr;

            *a_ptr = y;
            *b_ptr = x;
        };
    }

    #[inline(always)]
    pub fn store_jobj(&self, jobj: ObjectPtr, index: isize) {
        log::trace!(
            "store_jobj==addr : {:x?}==={:x?}",
            unsafe { self.bp.offset(-(index + 1)) },
            jobj.as_isize()
        );
        unsafe {
            (self.bp.offset(-(index + 1)) as *mut ObjectRawPtr).write(jobj.as_mut_raw_ptr());
        }
    }

    #[inline(always)]
    pub fn is_top_java_frame(&self) -> bool {
        return self.frame.is_java_top();
    }

    #[inline(always)]
    pub fn frame(&self) -> FramePtr {
        self.frame
    }

    pub fn stack_trace<F: FnMut(FramePtr)>(&self, mut action: F) {
        let mut frame = self.frame;
        while frame.is_not_null() {
            action(frame);
            frame = frame.prev();
        }
    }

    pub fn stack_trace_str(&self) -> String {
        let mut location = String::new();
        self.stack_trace(|frame| {
            location.push_str(
                format!(
                    "{}#{}\n",
                    frame.class().name().as_str(),
                    frame.method().name().as_str()
                )
                .as_str(),
            );
        });
        return location;
    }

    const fn calc_slots<T: StackPrimitiveValue>() -> isize {
        let val_size = std::mem::size_of::<T>();
        if val_size == 8 {
            return 2;
        }
        return 1;
    }
}

pub trait StackPrimitiveValue {}

impl StackPrimitiveValue for JFloat {}

impl StackPrimitiveValue for JDouble {}

impl StackPrimitiveValue for JInt {}

impl StackPrimitiveValue for JLong {}
