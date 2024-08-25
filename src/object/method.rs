use std::mem::size_of;

use crate::{
    define_oop,
    memory::{align, Address},
    thread::ThreadPtr,
};

use super::{array::JArrayPtr, class::JClassPtr, prelude::JInt, ptr::Ptr, symbol::SymbolPtr};

pub type MethodIndex = JInt;
pub type MethodPtr = Ptr<Method>;
pub type ExceptionTablePtr = Ptr<ExceptionTable>;

define_oop!(
    struct Method {
        decl_cls: JClassPtr,
        name: SymbolPtr,
        descriptor: SymbolPtr,
        params: JArrayPtr,
        ret_type: JClassPtr,
        ret_descriptor: SymbolPtr,
        access_flags: u16,
        max_stack: u16,
        max_locals: u16,
        code_length: u16,
        ex_tab_length: u16,
        native_fn: Address,
    }
);

impl Method {
    // pub const METHOD_CLASS: ClassPtr = ClassPtr::new(ObjectTag::Method as u8 as Address);

    pub fn new(
        access_flags: u16,
        name: SymbolPtr,
        descriptor: SymbolPtr,
        params: JArrayPtr,
        ret_type: JClassPtr,
        ret_descriptor: SymbolPtr,
        max_stack: u16,
        max_locals: u16,
        code_length: u16,
        code: *const u8,
        ex_tab: &Vec<ExceptionTable>,
        thread: ThreadPtr,
    ) -> MethodPtr {
        let mut method = MethodPtr::from_addr(
            thread
                .vm()
                .heap()
                .alloc_obj_permanent(Self::size(code_length, ex_tab.len() as u16)),
        );
        method.access_flags = access_flags;
        method.name = name;
        method.descriptor = descriptor;
        method.params = params;
        method.ret_type = ret_type;
        method.ret_descriptor = ret_descriptor;
        method.max_stack = max_stack;
        method.max_locals = max_locals;
        method.code_length = code_length;
        let method_code = method.code() as *mut u8;
        unsafe {
            std::ptr::copy(code, method_code, code_length as usize);
        }
        method.ex_tab_length = ex_tab.len() as u16;
        let method_ex_tab = method.ex_tab();
        unsafe {
            std::ptr::copy(
                ex_tab.as_ptr(),
                method_ex_tab.as_mut_raw_ptr(),
                method.ex_tab_length as usize,
            );
        }
        return method;
    }

    pub fn decl_cls(&self) -> JClassPtr {
        debug_assert!(self.decl_cls.is_not_null());
        return self.decl_cls;
    }

    pub fn set_decl_cls(&mut self, decl_cls: JClassPtr) {
        self.decl_cls = decl_cls;
    }

    pub fn decl_cls_opt(&self) -> Option<JClassPtr> {
        if self.decl_cls.is_not_null() {
            return Some(self.decl_cls);
        }
        return None;
    }

    pub fn name(&self) -> SymbolPtr {
        self.name
    }

    pub fn descriptor(&self) -> SymbolPtr {
        self.descriptor
    }

    pub fn params(&self) -> JArrayPtr {
        self.params
    }

    pub fn access_flags(&self) -> u16 {
        self.access_flags
    }

    pub fn is_public(&self) -> bool {
        return self.access_flags & (MethodAccessFlags::AccPublic as u16) != 0;
    }
    pub fn is_not_public(&self) -> bool {
        return self.access_flags & (MethodAccessFlags::AccPublic as u16) == 0;
    }

    pub fn is_private(&self) -> bool {
        return self.access_flags & (MethodAccessFlags::AccPrivate as u16) != 0;
    }

    pub fn is_protected(&self) -> bool {
        return self.access_flags & (MethodAccessFlags::AccProtected as u16) != 0;
    }

    pub fn is_abstract(&self) -> bool {
        return self.access_flags & (MethodAccessFlags::AccAbstract as u16) != 0;
    }

    pub fn is_static(&self) -> bool {
        return self.access_flags & (MethodAccessFlags::AccStatic as u16) != 0;
    }

    pub fn is_native(&self) -> bool {
        return self.access_flags & (MethodAccessFlags::AccNative as u16) != 0;
    }

    pub fn is_not_native(&self) -> bool {
        return self.access_flags & (MethodAccessFlags::AccNative as u16) == 0;
    }

    pub fn ret_type(&self) -> JClassPtr {
        self.ret_type
    }

    pub fn max_stack(&self) -> u16 {
        self.max_stack
    }

    pub fn set_max_stack(&mut self, max_stack: u16) {
        self.max_stack = max_stack;
    }

    pub fn max_locals(&self) -> u16 {
        self.max_locals
    }

    pub fn set_max_locals(&mut self, max_locals: u16) {
        self.max_locals = max_locals;
    }

    pub fn code_length(&self) -> u16 {
        self.code_length
    }

    pub fn code(&self) -> *const u8 {
        return Address::from_ref(self)
            .offset(size_of::<Self>() as isize)
            .raw_ptr();
    }

    pub fn ex_tab(&self) -> ExceptionTablePtr {
        return ExceptionTablePtr::from_addr(
            Address::from_ref(self).offset(Self::ex_tab_offset(self.code_length)),
        );
    }

    pub fn native_fn(&self) -> Address {
        self.native_fn
    }

    pub fn set_native_fn(&mut self, native_fn: Address) {
        self.native_fn = native_fn;
    }

    const fn size(code_length: u16, ex_tab_length: u16) -> usize {
        return (Self::ex_tab_offset(code_length)
            + size_of::<ExceptionTable>() as isize * ex_tab_length as isize)
            as usize;
    }

    const fn ex_tab_offset(code_length: u16) -> isize {
        return align(size_of::<Self>() + code_length as usize * size_of::<u8>()) as isize;
    }
}

pub struct ExceptionTable {
    pub(crate) start_pc: u16,
    pub(crate) end_pc: u16,
    pub(crate) handler_pc: u16,
    pub(crate) catch_type: u16,
}

impl ExceptionTable {
    pub fn new(start_pc: u16, end_pc: u16, handler_pc: u16, catch_type: u16) -> Self {
        return Self {
            start_pc,
            end_pc,
            handler_pc,
            catch_type,
        };
    }
}

pub enum MethodAccessFlags {
    AccPublic = 0x0001,
    AccPrivate = 0x0002,
    AccProtected = 0x0004,
    AccStatic = 0x0008,
    AccFinal = 0x0010,
    AccSynchronized = 0x0020,
    AccBridge = 0x0040,
    AccVarArgs = 0x0080,
    AccNative = 0x0100,
    AccAbstract = 0x0400,
    AccStrict = 0x0800,
    AccSynthetic = 0x1000,
}

pub struct ResolvedMethod {
    pub decl_class: JClassPtr,
    pub method: MethodPtr,
    pub method_idx: u32,
}
