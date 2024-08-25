use std::convert::TryFrom;

use crate::value::JValue;
use crate::{goto_label_addr, label, label_addr, load_reserved_value, reserve_value};

use crate::{
    memory::Address,
    object::{
        array::{
            JArray, JArrayPtr, JByteArrayPtr, JCharArrayPtr, JDoubleArrayPtr, JFloatArrayPtr,
            JIntArrayPtr, JLongArrayPtr, JShortArrayPtr,
        },
        class::{JClass, JClassPtr},
        constant_pool::ConstantTag,
        method::MethodPtr,
        prelude::{JByte, JChar, JDouble, JFloat, JInt, JLong, JShort, ObjectPtr},
        symbol::SymbolPtr,
        Object,
    },
    thread::{Thread, ThreadPtr},
    vm::VMPtr,
};

use paste::paste;

use super::stack::{Stack, StackPrimitiveValue};

macro_rules! jvm_instructions {
    (enum $name:ident {
        $($instr_name:ident = $instr_code:expr,)*
    }) => {
        #[allow(dead_code)]
        #[derive(Debug)]
        enum $name {
            $($instr_name = $instr_code),*
        }

        $(
            paste! {
                macro_rules! [<case_ label_ $instr_name:lower>] {
                    () => {
                        label!(stringify!([<__vm $instr_name:lower>]))
                    }
                }
            }
        )*

        impl $name {
            fn init_instructions_table(table: &mut [u64]) {
                $(
                    table[$instr_code] = label_addr!(paste! {
                        stringify!([<__vm $instr_name:lower>])
                    });
                )*
            }
        }
    };
}

jvm_instructions! {
    enum JvmInstruction {
        AALoad = 0x32,
        AAStore = 0x53,
        AConstNull = 0x1,
        ALoad = 0x19,
        ALoad0 = 0x2a,
        ALoad1 = 0x2b,
        ALoad2 = 0x2c,
        ALoad3 = 0x2d,
        ANewArray = 0xbd,
        AReturn = 0xb0,
        ArrayLength = 0xbe,
        AStore = 0x3a,
        AStore0 = 0x4b,
        AStore1 = 0x4c,
        AStore2 = 0x4d,
        AStore3 = 0x4e,
        AThrow = 0xbf,
        BALoad = 0x33,
        BAStore = 0x54,
        BIPush = 0x10,
        CALoad = 0x34,
        CAStore = 0x55,
        CheckCast = 0xc0,
        D2F = 0x90,
        D2I = 0x8e,
        D2L = 0x8f,
        DAdd = 0x63,
        DALoad = 0x31,
        DAStore = 0x52,
        DCmpG = 0x98,
        DCmpL = 0x97,
        DConst0 = 0xe,
        DConst1 = 0xf,
        DDiv = 0x6f,
        DLoad = 0x18,
        DLoad0 = 0x26,
        DLoad1 = 0x27,
        DLoad2 = 0x28,
        DLoad3 = 0x29,
        DMul = 0x6b,
        DNeg = 0x77,
        DRem = 0x73,
        DReturn = 0xaf,
        DStore = 0x39,
        DStore0 = 0x47,
        DStore1 = 0x48,
        DStore2 = 0x49,
        DStore3 = 0x4a,
        DSub = 0x67,
        Dup = 0x59,
        DupX1 = 0x5a,
        DupX2 = 0x5b,
        Dup2 = 0x5c,
        Dup2X1 = 0x5d,
        Dup2X2 = 0x5e,
        F2D = 0x8d,
        F2I = 0x8b,
        F2L = 0x8c,
        FAdd = 0x62,
        FALoad = 0x30,
        FAStore = 0x51,
        FCmpG = 0x96,
        FCmpL = 0x95,
        FConst0 = 0xb,
        FConst1 = 0xc,
        FConst2 = 0xd,
        FDiv = 0x6e,
        FLoad = 0x17,
        FLoad0 = 0x22,
        FLoad1 = 0x23,
        FLoad2 = 0x24,
        FLoad3 = 0x25,
        FMul = 0x6a,
        FNeg = 0x76,
        FRem = 0x72,
        FReturn = 0xae,
        FStore = 0x38,
        FStore0 = 0x43,
        FStore1 = 0x44,
        FStore2 = 0x45,
        FStore3 = 0x46,
        FSub = 0x66,
        GetField = 0xb4,
        GetStatic = 0xb2,
        Goto = 0xa7,
        GotoW = 0xc8,
        I2B = 0x91,
        I2C = 0x92,
        I2D = 0x87,
        I2F = 0x86,
        I2L = 0x85,
        I2S = 0x93,
        IAdd = 0x60,
        IALoad = 0x2e,
        IAnd = 0x7e,
        IAStore = 0x4f,
        IConstM1 = 0x2,
        IConst0 = 0x3,
        IConst1 = 0x4,
        IConst2 = 0x5,
        IConst3 = 0x6,
        IConst4 = 0x7,
        IConst5 = 0x8,
        IDiv = 0x6c,
        IfACmpEq = 0xa5,
        IfACmpNe = 0xa6,
        IfICmpEq = 0x9f,
        IfICmpNe = 0xa0,
        IfICmpLt = 0xa1,
        IfICmpGe = 0xa2,
        IfICmpGt = 0xa3,
        IfICmpLe = 0xa4,
        IfEq = 0x99,
        IfNe = 0x9a,
        IfLt = 0x9b,
        IfGe = 0x9c,
        IfGt = 0x9d,
        IfLe = 0x9e,
        IfNonNull = 0xc7,
        IfNull = 0xc6,
        IInc = 0x84,
        ILoad = 0x15,
        ILoad0 = 0x1a,
        ILoad1 = 0x1b,
        ILoad2 = 0x1c,
        ILoad3 = 0x1d,
        IMul = 0x68,
        INeg = 0x74,
        InstanceOf = 0xc1,
        InvokeDynamic = 0xba,
        InvokeInterface = 0xb9,
        InvokeSpecial = 0xb7,
        InvokeStatic = 0xb8,
        InvokeVirtual = 0xb6,
        IOr = 0x80,
        IRem = 0x70,
        IReturn = 0xac,
        IShL = 0x78,
        IShR = 0x7a,
        IStore = 0x36,
        IStore0 = 0x3b,
        IStore1 = 0x3c,
        IStore2 = 0x3d,
        IStore3 = 0x3e,
        ISub = 0x64,
        IUShR = 0x7c,
        IXor = 0x82,
        Jsr = 0xa8,  // jsr : Jump subroutine
        JsrW = 0xc9, // jsr_w : Jump subroutine (wide index)
        L2D = 0x8a,
        L2F = 0x89,
        L2I = 0x88,
        LAdd = 0x61,
        LALoad = 0x2f,
        LAnd = 0x7f,
        LAStore = 0x50,
        LCmp = 0x94,
        LConst0 = 0x9,
        LConst1 = 0xa,
        Ldc = 0x12,
        LdcW = 0x13,
        Ldc2W = 0x14,
        LDiv = 0x6d,
        LLoad = 0x16,
        LLoad0 = 0x1e,
        LLoad1 = 0x1f,
        LLoad2 = 0x20,
        LLoad3 = 0x21,
        LMul = 0x69,
        LNeg = 0x75,
        LookupSwitch = 0xab,
        LOr = 0x81,
        LRem = 0x71,
        LReturn = 0xad,
        LShL = 0x79,
        LShR = 0x7b,
        LStore = 0x37,
        LStore0 = 0x3f,
        LStore1 = 0x40,
        LStore2 = 0x41,
        LStore3 = 0x42,
        LSub = 0x65,
        LUShR = 0x7d,
        LXor = 0x83,
        MonitorEnter = 0xc2,
        MonitorExit = 0xc3,
        MultiANewArray = 0xc5,
        New = 0xbb,
        NewArray = 0xbc,
        Nop = 0x0,
        Pop = 0x57,
        Pop2 = 0x58,
        PutField = 0xb5,
        PutStatic = 0xb3,
        Ret = 0xa9,
        Return = 0xb1,
        SALoad = 0x35,
        SAStore = 0x56,
        SIPush = 0x11,
        Swap = 0x5f,
        TableSwitch = 0xaa,
        Wide = 0xc4,

        ImpDep1 = 0xfe,
        ImpDep2 = 0xff,

        Breakpoint = 0xca,
    }
}

macro_rules! case_label_array_load {
    ($op_code:ident, $arr: ty, ObjectRawPtr) => {{
        paste! {
            [<case_label_ $op_code>]!();
        }
        {
            let interp = access_interpreter!();
            let index = interp.stack.pop::<JInt>();
            let arr_ref: $arr = interp.stack.pop_jobj().cast();
            if arr_ref.is_null() {
                todo!("throw NullPointerException");
            }
            if index >= arr_ref.length() {
                log::trace!("outOfBounds {}, {}", arr_ref.length(), index);
                todo!("ArrayIndexOutOfBoundsException");
            }
            interp
                .stack
                .push_jobj(ObjectPtr::from_raw(arr_ref.get_raw(index)));
            dispatch!(interp);
        }
    }};
    ($op_code:ident, $arr: ty, $arr_ele: ty, $stack_ty: ty) => {{
        paste! {
            [<case_label_ $op_code>]!();
        }
        {
            let interp = access_interpreter!();
            let index = interp.stack.pop::<JInt>();
            let arr_ref: $arr = interp.stack.pop_jobj().cast();
            if arr_ref.is_null() {
                todo!("throw NullPointerException");
            }
            if index >= arr_ref.length() {
                todo!("ArrayIndexOutOfBoundsException");
            }
            interp.stack.push::<$stack_ty>(arr_ref.get(index) as _);
            dispatch!(interp);
        }
    }};
}

macro_rules! case_label_array_store {
    ($op_code:ident, $arr: ty, ObjectRawPtr) => {{
        paste! {
            [<case_label_ $op_code>]!();
        }
        {
            let interp = access_interpreter!();
            let val = interp.stack.pop_jobj();
            let index = interp.stack.pop::<JInt>();
            let arr_ref: $arr = interp.stack.pop_jobj().cast();
            if arr_ref.is_null() {
                todo!("throw NullPointerException")
            }
            if index >= arr_ref.length() {
                todo!("throw ArrayIndexOutOfBoundsException")
            }
            if !arr_ref.is_compatible(val, interp.vm) {
                todo!("throw ArrayStoreException")
            }
            arr_ref.set(index, val);
            dispatch!(interp);
        }
    }};
    ($op_code:ident, $arr: ty, $arr_ele: ty, $stack_ty: ty) => {{
        paste! {
            [<case_label_ $op_code>]!();
        }
        {
            let interp = access_interpreter!();
            let val: $arr_ele = interp.stack.pop::<$stack_ty>() as _;
            let index = interp.stack.pop::<JInt>();
            let arr_ref: $arr = interp.stack.pop_jobj().cast();
            if arr_ref.is_null() {
                todo!("throw NullPointerException")
            }
            if index >= arr_ref.length() {
                todo!("throw ArrayIndexOutOfBoundsException")
            }
            arr_ref.set(index, val);
            dispatch!(interp);
        }
    }};
}

macro_rules! case_label_val_store {
    ($op_code:ident, $index: expr, ObjectRawPtr) => {{
        paste! {
            [<case_label_ $op_code>]!();

            let interp = access_interpreter!();
            let index = $index;
            let val = interp.stack.pop_jobj();
            interp.stack.store_jobj(val, Interpreter::num2isize(index));
            dispatch!(interp);
        }
    }};
    ($op_code:ident, $index: expr, $val_ty: ty) => {{
        paste! {
            [<case_label_ $op_code>]!();

            let interp = access_interpreter!();
            let index = $index;
            let val = interp.stack.pop::<$val_ty>();
            interp.stack.store::<$val_ty>(val, Interpreter::num2isize(index));
            dispatch!(interp);
        }
    }};
}

macro_rules! case_label_num_arithmetic {
    ($op_code:ident, $val_ty: ty, $arith_op: tt, $divide_check: expr) => {{
        paste! {
            [<case_label_ $op_code>]!();

            let interp = access_interpreter!();
            let val2 = interp.stack.pop::<$val_ty>();
            let val1 = interp.stack.pop::<$val_ty>();
            if $divide_check && val2 == $val_ty::from(0u8) {
                todo!("throw ArithmeticException");
            }
            interp.stack.push::<$val_ty>(val1 $arith_op val2);
            dispatch!(interp);
        }
    }};
}

macro_rules! case_label_num_diff_types_arithmetic {
    ($op_code:ident, $val1_ty: ty, $val2_ty: ty, $arith_op: tt, $divide_check: expr) => {{
        paste! {
            [<case_label_ $op_code>]!();

            let interp = access_interpreter!();
            let val2 = interp.stack.pop::<$val2_ty>();
            let val1 = interp.stack.pop::<$val1_ty>();
            if $divide_check && val2 == $val2_ty::from(0u8) {
                todo!("throw ArithmeticException");
            }
            interp.stack.push::<$val1_ty>(val1 $arith_op val2);
            dispatch!(interp);
        }
    }};
}

macro_rules! case_label_num_convert {
    ($op_code:ident, $val_ty: ty, $to_val_ty:ty, $stack_ty: ty) => {{
        paste! {
            [<case_label_ $op_code>]!();

            let interp = access_interpreter!();
            let val = interp.stack.pop::<$val_ty>();
            interp.stack.push::<$stack_ty>((val as $to_val_ty) as $stack_ty);
            dispatch!(interp);
        }
    }};
}

macro_rules! case_label_num_load {
    ($op_code:ident, $val_ty: ty, $( $n:literal ),+) => {{
        paste! {
            [<case_label_ $op_code>]!();
            {
                let interp = access_interpreter!();
                let index = interp.read_operand();
                interp.op_val_load::<$val_ty>(index);
                dispatch!(interp);
            }

            ($(
                {
                    [<case_label_ $op_code $n>]!();
                    {
                        let interp = access_interpreter!();
                        interp.op_val_load::<$val_ty>($n);
                        dispatch!(interp);
                    }
                }
            ), +)
        }
    }};
}

macro_rules! case_label_num_const {
    ($op_code:ident, $val_ty: ty, $( $n:literal ),+) => {
        {
            paste! {
                ($(
                    {
                        [<case_label_ $op_code $n>]!();
                        {
                            let interp = access_interpreter!();
                            interp.stack.push::<$val_ty>($n);
                            dispatch!(interp);
                        }
                    }
                ), +)
            }
        }
    };
}

macro_rules! case_label_num_if_cmp {
    ($op_code:ident, $val_ty: ty, $val2_pop: ident, $arith_op: tt, $val1_pop: ident) => {{
        paste! {
            [<case_label_ $op_code>]!();
            {
                let interp = access_interpreter!();
                let if_op_addr = interp.pc.offset(-1);
                let branch = interp.read_operand_i16();
                let val2: $val_ty = interp.stack.$val2_pop();
                let val1: $val_ty = interp.stack.$val1_pop();
                if val1 $arith_op val2 {
                    interp.goto(if_op_addr, branch);
                }
                dispatch!(interp);
            }
        }
    }};
    ($op_code:ident, $val_ty: ty, $val2_pop: ident, $arith_op: tt, $val1: expr) => {{
        paste! {
            [<case_label_ $op_code>]!();
            {
                let interp = access_interpreter!();
                let if_op_addr = interp.pc.offset(-1);
                let branch = interp.read_operand_i16();
                let val2: $val_ty = interp.stack.$val2_pop();
                let val1: $val_ty = $val1;
                if val2 $arith_op val1 {
                    interp.goto(if_op_addr, branch);
                }
                dispatch!(interp);
            }
        }
    }};
}

macro_rules! do_num_load {
    ($val_ty: ty, $index: expr) => {{
        let interp = access_interpreter!();
        let val = interp.stack.load::<$val_ty>(Interpreter::num2isize($index));
        interp.stack.push::<$val_ty>(val);
    }};
}

macro_rules! do_num_store {
    ($val_ty: ty, $index: expr) => {{
        let interp = access_interpreter!();
        let val = interp.stack.pop::<$val_ty>();
        interp.stack.store(val, Interpreter::num2isize($index));
    }};
}

macro_rules! dispatch {
    ($interp: expr) => {
        let target_addr;
        unsafe {
            let op_code = *$interp.pc.raw_ptr();
            log::trace!(
                "opcode : 0x{:x} {:?} {:?}",
                op_code,
                Self::op_code_as_instr(op_code),
                $interp.pc.raw_ptr()
            );
            target_addr = OP_CODE_TABLE[usize::from(op_code)];
        }
        $interp.pc = $interp.pc.offset(std::mem::size_of::<u8>() as isize);
        reserve_value!($interp as *mut Self as usize);
        goto_label_addr!(target_addr);
    };
}

macro_rules! access_interpreter {
    () => {
        unsafe {
            // std::mem::transmute::<u64, &mut Interpreter>(load_reserved_value!())
            // interp
            //  &mut *(load_reserved_value!() as *mut Interpreter)
            &mut *(load_reserved_value!() as *mut Interpreter)
        }
    };
}

const OP_CODE_TABLE_SIZE: usize = 256;
static mut OP_CODE_TABLE: [u64; OP_CODE_TABLE_SIZE] = [0; OP_CODE_TABLE_SIZE];

pub struct Interpreter {
    thread: ThreadPtr,
    stack: Stack,
    pc: Address,
    vm: VMPtr,
}

impl Interpreter {
    pub fn new(stack_addr: Address, stack_size: usize, vm: VMPtr) -> Interpreter {
        JvmInstruction::init_instructions_table(unsafe { &mut OP_CODE_TABLE });
        let stack = Stack::new(stack_addr, stack_size);
        return Interpreter {
            thread: ThreadPtr::null(),
            stack,
            pc: Address::null(),
            vm,
        };
    }

    pub fn grand_parent_stack_class(&self) -> JClassPtr {
        let frame = self.stack.frame();
        if frame.is_not_null() && frame.has_prev() {
            let frame = frame.prev();
            if frame.is_not_null() && frame.has_prev() {
                return frame.prev().class();
            }
        }
        return JClassPtr::null();
    }

    pub fn call_static_method(
        class: JClassPtr,
        method: MethodPtr,
        args: &[JValue],
        thread: ThreadPtr,
    ) -> JValue {
        let interp = thread.as_mut_ref().interpreter_mut();
        interp.thread = thread;
        let params_len = method.params().length();
        let args_slots = {
            if params_len > 0 {
                let mut args_slots = 0;
                interp.prepare_args(ObjectPtr::null(), method, args, &mut args_slots);
                args_slots
            } else {
                0
            }
        };
        log::trace!(
            "call_static_void_method {}#{} code {:?}",
            class.name().as_str(),
            method.name().as_str(),
            method.code()
        );
        interp.invoke_method(
            ObjectPtr::null(),
            class,
            method,
            params_len as isize,
            args_slots,
            0,
            true,
        );
        interp.pc = Address::new(method.code());
        return Self::execute(interp);
    }

    pub fn call_obj_void_method(
        objref: ObjectPtr,
        method: MethodPtr,
        args: &[JValue],
        thread: ThreadPtr,
    ) {
        let interp = thread.as_mut_ref().interpreter_mut();
        interp.thread = thread;
        let args_slots = {
            let mut args_slots = 0;
            interp.prepare_args(objref, method, args, &mut args_slots);
            args_slots
        };
        log::trace!(
            "call_static_void_method {}#{} code {:?}",
            objref.jclass().name().as_str(),
            method.name().as_str(),
            method.code()
        );
        interp.invoke_method(
            objref,
            objref.jclass(),
            method,
            args.len() as isize,
            args_slots,
            1,
            true,
        );
        interp.pc = Address::new(method.code());
        Self::execute(interp);
    }

    pub fn call_obj_method(
        objref: ObjectPtr,
        method: MethodPtr,
        args: &[JValue],
        thread: ThreadPtr,
    ) -> JValue {
        debug_assert!(method.decl_cls().is_not_null());
        let interp = thread.as_mut_ref().interpreter_mut();
        interp.thread = thread;
        let args_slots = {
            let mut args_slots = 0;
            interp.prepare_args(objref, method, args, &mut args_slots);
            args_slots
        };
        interp.invoke_method(
            objref,
            method.decl_cls(),
            method,
            args.len() as isize,
            args_slots,
            1,
            true,
        );
        interp.pc = Address::new(method.code());
        return Self::execute(interp);
    }

    fn prepare_args(
        &mut self,
        objref: ObjectPtr,
        method: MethodPtr,
        args: &[JValue],
        args_slots: &mut isize,
    ) {
        let method_params = method.params();
        debug_assert_eq!(method_params.length(), args.len() as JInt);
        if !method.is_static() {
            *args_slots += 1;
            self.stack.push_jobj(objref);
        }
        let vm = self.vm;
        for param_index in 0..method_params.length() {
            let param_class: JClassPtr = method_params.get(param_index).cast();
            log::trace!("prepare_args param_class : 0x{:x}", param_class.as_isize());
            if JClass::is_long(param_class, vm) || JClass::is_double(param_class, vm) {
                let arg = unsafe { args.get_unchecked(param_index as usize).long_val() };
                self.stack.push::<JLong>(arg);
                *args_slots += 2;
            } else if param_class.is_not_null() && JClass::is_primitive(param_class) {
                let arg = unsafe { args.get_unchecked(param_index as usize).int_val() };
                self.stack.push::<JInt>(arg);
                *args_slots += 1;
            } else {
                let arg = unsafe { args.get_unchecked(param_index as usize).obj_val() };
                self.stack.push_jobj(arg);
                *args_slots += 1;
            }
        }
    }

    #[allow(dead_code)]
    fn execute(_interp: &mut Interpreter) -> JValue {
        debug_assert!(Thread::current().is_not_null());
        // let _scope = HandleScope::new(Thread::current());
        dispatch!(_interp);

        case_label_array_load!(aaload, JArrayPtr, ObjectRawPtr);
        case_label_array_store!(aastore, JArrayPtr, ObjectRawPtr);

        case_label_aconstnull!();
        {
            let interp = access_interpreter!();
            interp.stack.push_jobj(ObjectPtr::null());
            dispatch!(interp);
        }

        case_label_aload!();
        {
            let interp = access_interpreter!();
            let index = interp.read_operand();
            interp
                .stack
                .push_jobj(interp.stack.load_jobj(isize::from(index)));
            dispatch!(interp);
        }

        case_label_aload0!();
        {
            let interp = access_interpreter!();
            interp.stack.push_jobj(interp.stack.load_jobj(0));
            dispatch!(interp);
        }

        case_label_aload1!();
        {
            let interp = access_interpreter!();
            interp.stack.push_jobj(interp.stack.load_jobj(1));
            dispatch!(interp);
        }

        case_label_aload2!();
        {
            let interp = access_interpreter!();
            interp.stack.push_jobj(interp.stack.load_jobj(2));
            dispatch!(interp);
        }

        case_label_aload3!();
        {
            let interp = access_interpreter!();
            interp.stack.push_jobj(interp.stack.load_jobj(3));
            dispatch!(interp);
        }

        case_label_anewarray!();
        {
            let interp = access_interpreter!();
            let count = interp.stack.pop::<JInt>();
            if count < 0 {
                todo!("throw NegativeArraySizeException");
            }
            let index1 = u16::from(interp.read_operand());
            let index2 = u16::from(interp.read_operand());
            let cp_index = (index1 << 8) | index2;
            let component_cls_name = interp
                .stack
                .frame()
                .class()
                .class_data()
                .cp
                .get_class_name(cp_index);
            if component_cls_name.is_null() {
                todo!("Linking Exceptions")
            }
            // let array_class = interp
            //     .vm
            //     .bootstrap_class_loader
            //     .resolve_class(&format!("L{};", array_class_name.as_str()));
            let component_class = interp
                .vm
                .bootstrap_class_loader
                .load_class(component_cls_name.as_str());
            if let Ok(component_class) = component_class {
                let array_class = if component_class.class_data().is_array() {
                    interp
                        .vm
                        .bootstrap_class_loader
                        .load_class(&format!("[{}", component_cls_name.as_str()))
                } else {
                    interp
                        .vm
                        .bootstrap_class_loader
                        .load_class(&format!("[L{};", component_cls_name.as_str()))
                };
                match array_class {
                    Ok(array_cls) => {
                        let arr = JArray::new(count, array_cls, interp.thread);
                        // TODO
                        interp.stack.push_jobj(arr.cast());
                        dispatch!(interp);
                    }
                    Err(_e) => todo!("jvms-5.4.3.1. Class and Interface Resolution"),
                };
            } else {
                todo!("jvms-5.4.3.1. Class and Interface Resolution")
            }
        }

        case_label_areturn!();
        {
            let interp = access_interpreter!();
            let ret_val = interp.stack.pop_jobj();
            if interp.stack.is_top_java_frame() {
                interp.restore_invoker_frame();
                return JValue::with_obj_val(ret_val);
            }
            interp.restore_invoker_frame();
            interp.stack.push_jobj(ret_val);
            dispatch!(interp);
        }

        case_label_arraylength!();
        {
            let interp = access_interpreter!();
            let arr: JArrayPtr = interp.stack.pop_jobj().cast();
            interp.stack.push::<JInt>(arr.length());
            dispatch!(interp);
        }

        case_label_val_store!(astore, access_interpreter!().read_operand(), ObjectRawPtr);
        case_label_val_store!(astore0, 0, ObjectRawPtr);
        case_label_val_store!(astore1, 1, ObjectRawPtr);
        case_label_val_store!(astore2, 2, ObjectRawPtr);
        case_label_val_store!(astore3, 3, ObjectRawPtr);

        case_label_athrow!();
        {
            let interp = access_interpreter!();
            let ex = interp.stack.pop_jobj();
            if ex.is_null() {
                todo!("throw NullPointerException");
            }
            let frame_class = interp.stack.frame().class();
            if frame_class.is_not_null() {
                todo!("athrow not implemented!");
            }
        }

        case_label_array_load!(baload, JByteArrayPtr, JInt, JInt);

        case_label_array_store!(bastore, JByteArrayPtr, JByte, JInt);

        case_label_bipush!();
        {
            let interp = access_interpreter!();
            log::trace!("bipush haha {}", interp.stack.stack_trace_str());
            let val = JInt::from(interp.read_operand());
            interp.stack.push(val);
            dispatch!(interp);
        }

        case_label_array_load!(caload, JCharArrayPtr, JChar, JInt);

        case_label_array_store!(castore, JCharArrayPtr, JChar, JInt);

        case_label_checkcast!();
        {
            let interp = access_interpreter!();
            let index: u16 = u16::from(interp.read_operand());
            let index = (index << 8) | u16::from(interp.read_operand());
            let frame_class = interp.stack.frame().class();
            let ref_cls_name = frame_class.class_data().cp.get_class_name(index);
            let obj_ref = interp.stack.peek_jobj();
            if obj_ref.is_not_null() {
                match interp
                    .vm
                    .bootstrap_class_loader
                    .load_class(ref_cls_name.as_str())
                {
                    Ok(ref_cls) => {
                        if !ref_cls.is_assignable_from(obj_ref.jclass(), interp.vm) {
                            todo!("throw ClassCastException, ref_cls {}, obj_ref cls {}, stacktrace {}", ref_cls.name().as_str(), obj_ref.jclass().name().as_str(), interp.stack.stack_trace_str());
                        }
                    }
                    Err(_e) => todo!(),
                }
            }
            dispatch!(interp);
        }

        case_label_num_convert!(d2f, JDouble, JFloat, JFloat);
        case_label_num_convert!(d2i, JDouble, JInt, JInt);
        case_label_num_convert!(d2l, JDouble, JLong, JLong);

        case_label_num_arithmetic!(dadd, JDouble, +, false);

        case_label_array_load!(daload, JDoubleArrayPtr, JDouble, JDouble);

        case_label_array_store!(dastore, JDoubleArrayPtr, JDouble, JDouble);

        case_label_dcmpg!();
        {
            access_interpreter!().op_val_cmp::<JDouble>(1);
        }

        case_label_dcmpl!();
        {
            access_interpreter!().op_val_cmp::<JDouble>(-1);
        }

        case_label_dconst0!();
        {
            let interp = access_interpreter!();
            interp.stack.push::<JDouble>(0f64);
            dispatch!(interp);
        }

        case_label_dconst1!();
        {
            let interp = access_interpreter!();
            interp.stack.push::<JDouble>(1f64);
            dispatch!(interp);
        }

        case_label_num_arithmetic!(ddiv, JDouble, /, true);

        case_label_num_load!(dload, JDouble, 0, 1, 2, 3);

        case_label_num_arithmetic!(dmul, JDouble, *, false);

        case_label_dneg!();
        {
            let interp = access_interpreter!();
            let val = interp.stack.pop::<JDouble>();
            interp.stack.push::<JDouble>(-val);
            dispatch!(interp);
        }

        case_label_num_arithmetic!(drem, JDouble, %, true);

        case_label_dreturn!();
        {
            let interp = access_interpreter!();
            let ret_val = interp.stack.pop::<JDouble>();
            if interp.stack.is_top_java_frame() {
                interp.restore_invoker_frame();
                return JValue::with_double_val(ret_val);
            }
            interp.restore_invoker_frame();
            interp.stack.push::<JDouble>(ret_val);
            dispatch!(interp);
        }

        case_label_val_store!(dstore, access_interpreter!().read_operand(), JDouble);
        case_label_val_store!(dstore0, 0, JDouble);
        case_label_val_store!(dstore1, 1, JDouble);
        case_label_val_store!(dstore2, 2, JDouble);
        case_label_val_store!(dstore3, 3, JDouble);

        case_label_num_arithmetic!(dsub, JDouble, -, false);

        case_label_dup!();
        {
            let interp = access_interpreter!();
            interp.stack.push_slot(interp.stack.peek_slot());
            dispatch!(interp);
        }

        case_label_dupx1!();
        {
            let interp = access_interpreter!();
            let val1 = interp.stack.pop_slot();
            let val2 = interp.stack.pop_slot();
            interp.stack.push_slot(val1);
            interp.stack.push_slot(val2);
            interp.stack.push_slot(val1);
            dispatch!(interp);
        }

        case_label_dupx2!();
        {
            let interp = access_interpreter!();
            let val1 = interp.stack.pop_slot();
            let val2 = interp.stack.pop::<JLong>();
            interp.stack.push_slot(val1);
            interp.stack.push(val2);
            interp.stack.push_slot(val1);
            dispatch!(interp);
        }

        case_label_dup2!();
        {
            let interp = access_interpreter!();
            let val = interp.stack.pop::<JLong>();
            interp.stack.push(val);
            interp.stack.push(val);
            dispatch!(interp);
        }

        case_label_dup2x1!();
        {
            let interp = access_interpreter!();
            let val1 = interp.stack.pop::<JLong>();
            let val2 = interp.stack.pop_slot();
            interp.stack.push(val1);
            interp.stack.push_slot(val2);
            interp.stack.push(val1);
            dispatch!(interp);
        }

        case_label_dup2x2!();
        {
            let interp = access_interpreter!();
            let val1 = interp.stack.pop::<JLong>();
            let val2 = interp.stack.pop::<JLong>();
            interp.stack.push(val1);
            interp.stack.push(val2);
            interp.stack.push(val1);
            dispatch!(interp);
        }

        case_label_num_convert!(f2d, JFloat, JDouble, JDouble);
        case_label_num_convert!(f2i, JFloat, JInt, JInt);
        case_label_num_convert!(f2l, JFloat, JLong, JLong);

        case_label_num_arithmetic!(fadd, JFloat, +, false);

        case_label_array_load!(faload, JFloatArrayPtr, JFloat, JFloat);

        case_label_array_store!(fastore, JFloatArrayPtr, JFloat, JFloat);

        case_label_fcmpg!();
        {
            access_interpreter!().op_val_cmp::<JFloat>(1);
        }

        case_label_fcmpl!();
        {
            access_interpreter!().op_val_cmp::<JFloat>(-1);
        }

        case_label_fconst0!();
        {
            let interp = access_interpreter!();
            interp.stack.push::<JFloat>(0f32);
            dispatch!(interp);
        }

        case_label_fconst1!();
        {
            let interp = access_interpreter!();
            interp.stack.push::<JFloat>(1f32);
            dispatch!(interp);
        }

        case_label_fconst2!();
        {
            let interp = access_interpreter!();
            interp.stack.push::<JFloat>(2f32);
            dispatch!(interp);
        }

        case_label_num_arithmetic!(fdiv, JFloat, /, true);

        case_label_num_load!(fload, JFloat, 0, 1, 2, 3);

        case_label_num_arithmetic!(fmul, JFloat, *, false);

        case_label_fneg!();
        {
            let interp = access_interpreter!();
            let val = interp.stack.pop::<JFloat>();
            interp.stack.push::<JFloat>(-val);
            dispatch!(interp);
        }

        case_label_num_arithmetic!(frem, JFloat, %, true);

        case_label_freturn!();
        {
            let interp = access_interpreter!();
            let ret_val = interp.stack.pop::<JFloat>();
            if interp.stack.is_top_java_frame() {
                interp.restore_invoker_frame();
                return JValue::with_float_val(ret_val);
            }
            interp.restore_invoker_frame();
            interp.stack.push::<JFloat>(ret_val);
            dispatch!(interp);
        }

        case_label_val_store!(fstore, access_interpreter!().read_operand(), JFloat);
        case_label_val_store!(fstore0, 0, JFloat);
        case_label_val_store!(fstore1, 1, JFloat);
        case_label_val_store!(fstore2, 2, JFloat);
        case_label_val_store!(fstore3, 3, JFloat);

        case_label_num_arithmetic!(fsub, JFloat, -, false);

        case_label_getfield!(); // jvms-5.4.3.2
        {
            let interp = access_interpreter!();
            let index = u16::from(interp.read_operand());
            let index = (index << 8u16) | u16::from(interp.read_operand());
            let obj = interp.stack.pop_jobj();
            if obj.is_null() {
                todo!(
                    "throws NullPointerException \n {}",
                    interp.stack.stack_trace_str()
                );
            }
            let frame_cls = interp.stack.frame().method().decl_cls();
            let thread = Thread::current();
            let field_ref = frame_cls.class_data().cp.get_field_ref(index);
            let field_lookup_cls: JClassPtr;
            if field_ref.class_name == frame_cls.name() {
                field_lookup_cls = frame_cls;
            } else {
                if let Ok(loaded_field_cls) = interp
                    .vm
                    .bootstrap_class_loader
                    .load_class(field_ref.class_name.as_str())
                {
                    field_lookup_cls = loaded_field_cls;
                } else {
                    todo!();
                }
            }
            let (field, _) = field_lookup_cls.get_field(&field_ref);
            let field_value = match field.get_value(obj, thread) {
                Ok(field_value) => field_value,
                Err(_e) => todo!(),
            };
            log::trace!(
                "get field ====== {}.{}, obj: 0x{:x}, val: 0x{:x}, offset {}, stacktrace: {}",
                field_lookup_cls.name().as_str(),
                field_ref.member_name.as_str(),
                obj.as_isize(),
                field_value,
                field.layout_offset(),
                interp.stack.stack_trace_str(),
            );
            let field_cls = field.field_class_unchecked();
            let vm = interp.vm;
            if JClass::is_long(field_cls, vm) || JClass::is_double(field_cls, vm) {
                interp.stack.push::<JLong>(field_value);
            } else if JClass::is_primitive(field_cls) {
                interp.stack.push::<JInt>(field_value as JInt);
            } else {
                interp
                    .stack
                    .push_jobj(ObjectPtr::from_isize(Interpreter::num2isize(field_value)));
            }
            dispatch!(interp);
        }

        case_label_getstatic!();
        {
            let interp = access_interpreter!();
            let index = u16::from(interp.read_operand());
            let index = (index << 8u16) | u16::from(interp.read_operand());
            let frame_class = interp.stack.frame().class();
            let field_ref = frame_class.class_data().cp.get_field_ref(index);
            let vm = interp.vm;
            if let Ok(_resolved_class) = vm
                .bootstrap_class_loader
                .load_class(field_ref.class_name.as_str())
            {
                let thread = Thread::current();
                let (field, decl_cls) = _resolved_class.get_field(&field_ref);
                match decl_cls.initialize(thread) {
                    Ok(_) => {}
                    Err(_) => todo!(),
                }
                let field_class = field.field_class_unchecked();
                log::trace!(
                    "getstatic {}#{} : cls 0x{:x}   success, offset: {}",
                    decl_cls.name().as_str(),
                    field.name().as_str(),
                    decl_cls.as_isize(),
                    field.layout_offset()
                );
                if JClass::is_long(field_class, vm) || JClass::is_double(field_class, vm) {
                    log::trace!(
                        "getstatic {}#{} , val {}",
                        decl_cls.name().as_str(),
                        field.name().as_str(),
                        field.get_static_value(decl_cls)
                    );
                    interp.stack.push::<JLong>(field.get_static_value(decl_cls));
                } else if JClass::is_primitive(field_class) {
                    interp
                        .stack
                        .push::<JInt>(field.get_static_value(decl_cls) as JInt);
                } else {
                    let value = field.get_static_value(decl_cls);
                    log::trace!(
                        "getstatic {}#{} : cls 0x{:x}, val 0x{:x?} success, offset: {}",
                        decl_cls.name().as_str(),
                        field.name().as_str(),
                        decl_cls.as_isize(),
                        value,
                        field.layout_offset()
                    );
                    interp
                        .stack
                        .push_jobj(ObjectPtr::from_isize(Interpreter::num2isize(
                            field.get_static_value(decl_cls),
                        )));
                }
            } else {
                todo!("throw ClassNotFoundException");
            }
            dispatch!(interp);
        }

        case_label_goto!();
        {
            let interp = access_interpreter!();
            let goto_op_addr = interp.pc.offset(-1);
            let branch = interp.read_operand_i16();
            interp.goto(goto_op_addr, branch);
        }

        case_label_gotow!();
        {
            let interp = access_interpreter!();
            let gotow_op_addr = interp.pc.offset(-1);
            let branch = interp.read_operand_i32();
            interp.goto_w(gotow_op_addr, branch);
        }

        case_label_num_convert!(i2b, JInt, JByte, JInt);
        case_label_num_convert!(i2c, JInt, JChar, JInt);
        case_label_num_convert!(i2d, JInt, JDouble, JDouble);
        case_label_num_convert!(i2f, JInt, JFloat, JFloat);
        case_label_num_convert!(i2l, JInt, JLong, JLong);
        case_label_num_convert!(i2s, JInt, JShort, JInt);

        case_label_num_arithmetic!(iadd, JInt, +, false);

        case_label_array_load!(iaload, JIntArrayPtr, JInt, JInt);

        case_label_num_arithmetic!(iand, JInt, &, false);

        case_label_array_store!(iastore, JIntArrayPtr, JInt, JInt);

        case_label_iconstm1!();
        {
            let interp = access_interpreter!();
            interp.stack.push::<JInt>(-1);
            dispatch!(interp);
        }

        case_label_num_const!(iconst, JInt, 0, 1, 2, 3, 4, 5);

        case_label_num_arithmetic!(idiv, JInt, /, true);

        case_label_num_if_cmp!(ifacmpeq, ObjectPtr, pop_jobj, ==, pop_jobj);

        case_label_num_if_cmp!(ifacmpne, ObjectPtr, pop_jobj, !=, pop_jobj);

        case_label_num_if_cmp!(ificmpeq, JInt, pop, ==, pop);
        case_label_num_if_cmp!(ificmpne, JInt, pop, !=, pop);
        case_label_num_if_cmp!(ificmplt, JInt, pop,  <, pop);
        case_label_num_if_cmp!(ificmpge, JInt, pop, >=, pop);
        case_label_num_if_cmp!(ificmpgt, JInt, pop, >, pop);
        case_label_num_if_cmp!(ificmple, JInt, pop, <=, pop);

        case_label_num_if_cmp!(ifeq, JInt, pop, ==, 0);
        case_label_num_if_cmp!(ifne, JInt, pop, !=, 0);
        case_label_num_if_cmp!(iflt, JInt, pop, <, 0);
        case_label_num_if_cmp!(ifge, JInt, pop, >=, 0);
        case_label_num_if_cmp!(ifgt, JInt, pop, >, 0);
        case_label_num_if_cmp!(ifle, JInt, pop, <=, 0);

        case_label_num_if_cmp!(ifnonnull, ObjectPtr, pop_jobj, !=, ObjectPtr::null());
        case_label_num_if_cmp!(ifnull, ObjectPtr, pop_jobj, ==, ObjectPtr::null());

        case_label_iinc!();
        {
            let interp = access_interpreter!();
            let index = interp.read_operand();
            let const_val = JInt::from(interp.read_op::<i8>());
            log::trace!(
                "iincc index {}, raw: {}, const_val: {}",
                index,
                interp.stack.load::<JInt>(isize::from(index)),
                const_val
            );
            interp.stack.iinc(const_val, isize::from(index));
            dispatch!(interp);
        }

        case_label_num_load!(iload, JInt, 0, 1, 2, 3);

        case_label_num_arithmetic!(imul, JInt, *, false);

        case_label_ineg!();
        {
            let interp = access_interpreter!();
            let val = interp.stack.pop::<JInt>();
            interp.stack.push::<JInt>(-val);
            dispatch!(interp);
        }

        case_label_instanceof!();
        {
            let interp = access_interpreter!();
            let index = u16::from(interp.read_operand());
            let index = (index << 8) | u16::from(interp.read_operand());
            let obj_ref = interp.stack.pop_jobj();
            if obj_ref.is_null() {
                interp.stack.push::<JInt>(0);
                dispatch!(interp);
            }
            let frame_class = interp.stack.frame().class();
            let target_class_name = frame_class.class_data().cp.get_class_name(index);
            if let Ok(target_class) = interp
                .vm
                .bootstrap_class_loader
                .load_class(target_class_name.as_str())
            {
                if obj_ref.is_instance_of(target_class, interp.vm) {
                    interp.stack.push::<JInt>(1);
                } else {
                    interp.stack.push::<JInt>(0);
                }
            } else {
                todo!("throw ClassNotFoundException");
            }
            dispatch!(interp);
        }

        case_label_invokedynamic!();
        {
            let interp = access_interpreter!();
            let index = u16::from(interp.read_operand());
            let index = (index << 8) | u16::from(interp.read_operand());
            interp.read_operand();
            interp.read_operand();
            if index < 0xff {
                todo!();
            }
            dispatch!(interp);
        }

        case_label_invokeinterface!();
        {
            let interp = access_interpreter!();
            let index = u16::from(interp.read_operand());
            let index = (index << 8) | u16::from(interp.read_operand());
            let args_slots = isize::from(interp.read_operand());
            if args_slots <= 0 {
                todo!("throw InvalidFormatException");
            }
            interp.read_operand();
            let objref = interp.stack.load_callee_objref(args_slots);
            if objref.is_null() {
                todo!("throw NullPointerException");
            }
            let frame_class = interp.stack.frame().class();
            log::trace!(
                "invokeinterface frame class {}, index {}, objref class {}",
                frame_class.name().as_str(),
                index,
                objref.jclass().name().as_str()
            );
            let member_ref = frame_class.class_data().cp.get_interface_method_ref(index);
            if let Ok(if_class) = interp
                .vm
                .bootstrap_class_loader
                .load_class(member_ref.class_name.as_str())
            {
                match JClass::resolve_interface_method(
                    objref.jclass(),
                    if_class,
                    member_ref.member_name,
                    member_ref.member_desc,
                ) {
                    Ok(resolved_method) => {
                        let target_method = resolved_method.method;
                        if !target_method.is_public() {
                            todo!("throw IllegalAccessError");
                        }
                        if target_method.is_abstract() {
                            log::trace!(
                                "invokeinterface class {}, objref addr 0x{:x}, method: {}, method addr 0x{:x}, descriptor: {}",
                                objref.jclass().name().as_str(),
                                objref.as_isize(),
                                member_ref.member_name.as_str(),
                                target_method.as_isize(),
                                member_ref.member_desc.as_str(),
                            );
                            JClass::debug(objref.jclass());
                            todo!("throw AbstractMethodError");
                        }
                        interp.invoke_method(
                            objref,
                            objref.jclass(),
                            target_method,
                            target_method.params().length() as isize,
                            args_slots,
                            1,
                            false,
                        );
                        dispatch!(interp);
                    }
                    Err(e) => todo!("{:#?}", e),
                }
            } else {
                todo!("throw ClassNotFound");
            }
        }

        case_label_invokespecial!();
        {
            let interp = access_interpreter!();
            let index = interp.read_operand_u16();

            let frame_class = interp.stack.frame().class();
            let member_ref = frame_class.class_data().cp.get_method_ref(index);
            let (resolved_method, target_cls) = if member_ref.class_name == frame_class.name() {
                match frame_class
                    .resolve_self_method(member_ref.member_name, member_ref.member_desc)
                {
                    Ok(resolved_method) => (resolved_method.method, frame_class),
                    Err(_e) => todo!(),
                }
            } else {
                if let Ok(target_class) = interp
                    .vm
                    .bootstrap_class_loader
                    .load_class(member_ref.class_name.as_str())
                {
                    if target_class.class_data().is_interface() {
                        todo!("throw IncompatibleClassChangeError");
                    }
                    match target_class.resolve_class_method(
                        member_ref.member_name,
                        member_ref.member_desc,
                        interp.vm.as_ref(),
                    ) {
                        Ok(resolved_method) => {
                            let resolved_method = resolved_method.method;
                            (resolved_method, resolved_method.decl_cls())
                        }
                        Err(_e) => todo!(),
                    }
                } else {
                    todo!("throw ClassNotFoundException");
                }
            };
            log::trace!(
                "case_label_invokespecial resolved method name {}::{}",
                target_cls.name().as_str(),
                resolved_method.name().as_str()
            );
            let args_count = isize::try_from(resolved_method.params().length()).unwrap();
            let args_slots = 1 + interp.compute_args_slots(resolved_method, interp.vm);
            let objref = interp.stack.load_callee_objref(args_slots);
            if objref.is_null() {
                todo!("throw NullPointerException");
            }
            interp.invoke_method(
                objref,
                target_cls,
                resolved_method,
                args_count,
                args_slots,
                1,
                false,
            );
            dispatch!(interp);
        }

        case_label_invokestatic!();
        {
            let interp = access_interpreter!();
            let index = u16::from(interp.read_operand());
            let index = (index << 8) | u16::from(interp.read_operand());
            let frame_class = interp.stack.frame().class();
            log::trace!(
                "invokestatic {}#{}, index {}, stacktrace {}",
                frame_class.name().as_str(),
                interp.stack.frame().method().name().as_str(),
                index,
                interp.stack.stack_trace_str()
            );
            let member_ref = frame_class.class_data().cp.get_method_ref(index);
            if let Ok(target_class) = interp
                .vm
                .bootstrap_class_loader
                .load_class(member_ref.class_name.as_str())
            {
                if target_class.class_data().is_interface() {
                    todo!("throw IncompatibleClassChangeError");
                }
                match target_class.initialize(Thread::current()) {
                    Ok(_) => {}
                    Err(_) => todo!(),
                }
                match target_class
                    .resolve_self_method(member_ref.member_name, member_ref.member_desc)
                {
                    Ok(resolved_method) => {
                        let resolved_method = resolved_method.method;
                        if !resolved_method.is_static() {
                            todo!("throw IncompatibleClassChangeError");
                        }
                        let args_count = Self::num2isize(resolved_method.params().length());
                        let args_slots = interp.compute_args_slots(resolved_method, interp.vm);
                        interp.invoke_method(
                            ObjectPtr::null(),
                            target_class,
                            resolved_method,
                            args_count,
                            args_slots,
                            0,
                            false,
                        );
                        dispatch!(interp);
                    }
                    Err(_) => todo!(),
                }
            } else {
                todo!("throw ClassNotFoundException");
            }
        }

        case_label_invokevirtual!();
        {
            let interp = access_interpreter!();
            let index = u16::from(interp.read_operand());
            let index = (index << 8) | u16::from(interp.read_operand());
            let frame_class = interp.stack.frame().class();
            let member_ref = frame_class.class_data().cp.get_method_ref(index);
            log::trace!(
                "invokvirtual from {}#{}, target {}#{}, index {}, stacktrace {}",
                frame_class.name().as_str(),
                interp.stack.frame().method().name().as_str(),
                member_ref.class_name.as_str(),
                member_ref.member_name.as_str(),
                index,
                interp.stack.stack_trace_str()
            );
            match interp
                .vm
                .bootstrap_class_loader
                .load_class(member_ref.class_name.as_str())
            {
                Ok(target_class) => {
                    if target_class.class_data().is_interface() {
                        todo!("throw IncompatibleClassChangeError");
                    }
                    match target_class.resolve_class_method(
                        member_ref.member_name,
                        member_ref.member_desc,
                        interp.vm.as_ref(),
                    ) {
                        Ok(resolved_method) => {
                            if resolved_method.method.is_static() {
                                todo!("throw IncompatibleClassChangeError");
                            }
                            let args_count =
                                Self::num2isize(resolved_method.method.params().length());
                            let args_slots =
                                1 + interp.compute_args_slots(resolved_method.method, interp.vm);
                            let obj_ref = interp.stack.load_callee_objref(args_slots);
                            match JClass::resolve_virtual_with_index(
                                obj_ref,
                                resolved_method.method,
                                resolved_method.method_idx,
                            ) {
                                Ok(resolved_method) => {
                                    log::trace!("invokvirtual obj_ref 0x{:x}", obj_ref.as_isize());
                                    interp.invoke_method(
                                        obj_ref,
                                        resolved_method.method.decl_cls(),
                                        resolved_method.method,
                                        args_count,
                                        args_slots,
                                        1,
                                        false,
                                    );
                                    dispatch!(interp);
                                }
                                Err(_e) => {
                                    log::trace!("invokevirtual failed {:?}", _e);
                                    todo!();
                                }
                            };
                        }
                        Err(_) => todo!(),
                    }
                }
                Err(e) => {
                    log::trace!(
                        "class not found: {}, e: {:#?}",
                        member_ref.class_name.as_str(),
                        e
                    );
                    todo!("throw ClassNotFoundException");
                }
            }
        }

        case_label_num_arithmetic!(ior, JInt, |, false);
        case_label_num_arithmetic!(irem, JInt, %, true);

        case_label_ireturn!();
        {
            let interp = access_interpreter!();
            let ret_val = interp.stack.pop::<JInt>();
            if interp.stack.is_top_java_frame() {
                interp.restore_invoker_frame();
                return JValue::with_int_val(ret_val);
            }
            interp.restore_invoker_frame();
            interp.stack.push::<JInt>(ret_val);
            dispatch!(interp);
        }

        case_label_num_arithmetic!(ishl, JInt, <<, false);
        case_label_num_arithmetic!(ishr, JInt, >>, false);

        case_label_val_store!(istore, access_interpreter!().read_operand(), JInt);
        case_label_val_store!(istore0, 0, JInt);
        case_label_val_store!(istore1, 1, JInt);
        case_label_val_store!(istore2, 2, JInt);
        case_label_val_store!(istore3, 3, JInt);

        case_label_num_arithmetic!(isub, JInt, -, false);

        case_label_iushr!();
        {
            let interp = access_interpreter!();
            let val2 = interp.stack.pop::<JInt>();
            let val1 = interp.stack.pop::<JInt>();
            if val1 > 0 {
                interp.stack.push::<JInt>(val1 >> (val2 & 0x1f));
            } else if val1 < 0 {
                interp
                    .stack
                    .push::<JInt>((val1 >> (val2 & 0x1f)) + (2 << !(val2 & 0x1f)));
            } else {
                interp.stack.push::<JInt>(0);
            }
            dispatch!(interp);
        }

        case_label_num_arithmetic!(ixor, JInt, ^, false);

        case_label_jsr!();
        {
            let interp = access_interpreter!();
            let jsr_op_addr = interp.pc.offset(-1);
            let branch = i16::from(interp.read_operand());
            let branch = (branch << 8) | i16::from(interp.read_operand());
            interp.stack.push_jobj(ObjectPtr::from_addr(interp.pc));
            interp.goto(jsr_op_addr, branch);
        }

        case_label_jsrw!();
        {
            let interp = access_interpreter!();
            let jsrw_op_addr = interp.pc.offset(-1);
            let branch = i32::from(interp.read_operand()) << 24;
            let branch = branch | (i32::from(interp.read_operand()) << 16);
            let branch = branch | (i32::from(interp.read_operand()) << 8);
            let branch = branch | i32::from(interp.read_operand());
            interp.stack.push_jobj(ObjectPtr::from_addr(interp.pc));
            interp.goto_w(jsrw_op_addr, branch);
        }

        case_label_num_convert!(l2d, JLong, JDouble, JDouble);
        case_label_num_convert!(l2f, JLong, JFloat, JFloat);
        case_label_num_convert!(l2i, JLong, JInt, JInt);

        case_label_num_arithmetic!(ladd, JLong, +, false);

        case_label_array_load!(laload, JLongArrayPtr, JLong, JLong);

        case_label_num_arithmetic!(land, JLong, &, false);

        case_label_array_store!(lastore, JLongArrayPtr, JLong, JLong);

        case_label_lcmp!();
        {
            access_interpreter!().op_val_cmp::<JLong>(0);
        }

        case_label_num_const!(lconst, JLong, 0, 1);

        case_label_ldc!();
        {
            let interp = access_interpreter!();
            let index = u16::from(interp.read_operand());
            Self::op_ldc(interp, index);
            dispatch!(interp);
        }

        case_label_ldcw!();
        {
            let interp = access_interpreter!();
            let index = u16::from(interp.read_operand());
            let index = (index << 8) | u16::from(interp.read_operand());
            Self::op_ldc(interp, index);
            dispatch!(interp);
        }

        case_label_ldc2w!();
        {
            let interp = access_interpreter!();
            let index = u16::from(interp.read_operand());
            let index = (index << 8) | u16::from(interp.read_operand());
            let frame_class = interp.stack.frame().class();
            let constant_tag = frame_class.class_data().cp.get_tag(index);
            match constant_tag {
                ConstantTag::Long => {
                    interp
                        .stack
                        .push(frame_class.class_data().cp.get_long(index));
                }
                ConstantTag::Double => {
                    interp
                        .stack
                        .push(frame_class.class_data().cp.get_double(index));
                }
                _ => {
                    todo!("invalid constant tag");
                }
            }
            dispatch!(interp);
        }

        case_label_num_arithmetic!(ldiv, JLong, /, true);

        case_label_num_load!(lload, JLong, 0, 1, 2, 3);

        case_label_num_arithmetic!(lmul, JLong, *, false);

        case_label_lneg!();
        {
            let interp = access_interpreter!();
            let val = interp.stack.pop::<JLong>();
            interp.stack.push::<JLong>(-val);
            dispatch!(interp);
        }

        case_label_lookupswitch!();
        {
            let interp = access_interpreter!();
            let op_addr = interp.pc.offset(-1);
            interp.skip_operands(3);
            let default_offset = interp.read_operand_i32();

            let npairs = interp.read_operand_i32();

            let key = interp.stack.pop::<JInt>();

            let mut left = 0;
            let mut right = npairs - 1;

            while left <= right {
                let mid = (left + right) / 2;
                let mid_val = interp.peek_operand_as_int(Self::num2isize(mid * 8));
                if mid_val < key {
                    left = mid + 1;
                } else if mid_val > key {
                    right = mid - 1;
                } else {
                    let offset = interp.peek_operand_as_int(Self::num2isize(mid * 8) + 4);
                    interp.pc = op_addr.offset(Self::num2isize(offset));
                    dispatch!(interp);
                }
            }
            interp.pc = op_addr.offset(Self::num2isize(default_offset));
            dispatch!(interp);
        }

        case_label_num_arithmetic!(lor, JLong, |, false);
        case_label_num_arithmetic!(lrem, JLong, %, true);

        case_label_lreturn!();
        {
            let interp = access_interpreter!();
            let ret_val = interp.stack.pop::<JLong>();
            if interp.stack.is_top_java_frame() {
                interp.restore_invoker_frame();
                return JValue::with_long_val(ret_val);
            }
            interp.restore_invoker_frame();
            interp.stack.push::<JLong>(ret_val);
            dispatch!(interp);
        }

        case_label_num_diff_types_arithmetic!(lshl, JLong, JInt, <<, false);
        case_label_num_diff_types_arithmetic!(lshr, JLong, JInt, >>, false);

        case_label_val_store!(lstore, access_interpreter!().read_operand(), JLong);
        case_label_val_store!(lstore0, 0, JLong);
        case_label_val_store!(lstore1, 1, JLong);
        case_label_val_store!(lstore2, 2, JLong);
        case_label_val_store!(lstore3, 3, JLong);

        case_label_num_arithmetic!(lsub, JLong, -, false);

        case_label_lushr!();
        {
            let interp = access_interpreter!();
            let val2 = interp.stack.pop::<JInt>();
            let val1 = interp.stack.pop::<JLong>();
            if val1 > 0 {
                interp.stack.push::<JLong>(val1 >> (val2 & 0x3f));
            } else if val1 < 0 {
                interp
                    .stack
                    .push::<JLong>((val1 >> (val2 & 0x3f)) + (2i64 << !(val2 & 0x3f)));
            } else {
                interp.stack.push::<JLong>(0);
            }
            dispatch!(interp);
        }

        case_label_num_arithmetic!(lxor, JLong, ^, false);

        case_label_monitorenter!();
        {
            let interp = access_interpreter!();
            let obj = interp.stack.pop_jobj();
            if obj.is_null() {
                todo!("throw NullPointerException");
            }
            // TODO
            dispatch!(interp);
        }

        case_label_monitorexit!();
        {
            // todo!();
            let interp = access_interpreter!();
            let obj = interp.stack.pop_jobj();
            if obj.is_null() {
                todo!("throw NullPointerException");
            }
            // TODO
            dispatch!(interp);
        }

        case_label_multianewarray!();
        {
            let interp = access_interpreter!();
            let index = u16::from(interp.read_operand());
            let index = (index << 8) | u16::from(interp.read_operand());
            let dimensions = interp.read_operand();
            if dimensions < 1 {
                todo!("throw ClassFormatError");
            }
            let dimensions_class_name = interp
                .stack
                .frame()
                .class()
                .class_data()
                .cp
                .get_class_name(index);
            if let Ok(dimension_class) = interp
                .vm
                .bootstrap_class_loader
                .load_class(dimensions_class_name.as_str())
            {
                let dimensions_end_idx = dimensions - 1;
                let dimension_length = interp.stack.peek_int(dimensions_end_idx as isize);
                let dimensions_array =
                    JArray::new(dimension_length, dimension_class, interp.thread);
                if dimensions > 1 {
                    interp.create_dimension_array(
                        1,
                        dimensions_end_idx,
                        dimensions_class_name,
                        dimensions_array,
                        dimension_class.class_data().component_type(),
                    );
                }
                interp.stack.push_jobj(dimensions_array.cast());

                dispatch!(interp);
            } else {
                todo!();
            }
        }

        case_label_new!();
        {
            let interp = access_interpreter!();
            let index = interp.read_operand_u16();
            let target_class_name = interp
                .stack
                .frame()
                .class()
                .class_data()
                .cp
                .get_class_name(index);
            if let Ok(target_class) = interp
                .vm
                .bootstrap_class_loader
                .load_class(target_class_name.as_str())
            {
                match target_class.initialize(Thread::current()) {
                    Ok(_) => {}
                    Err(_) => todo!(),
                }
                let obj = Object::new(target_class, interp.thread);
                log::trace!(
                    "case_label_new {}, obj addr {:x}, obj inst size: {}, name addr {:x}",
                    obj.jclass().name().as_str(),
                    obj.as_usize(),
                    target_class.class_data().inst_or_ele_size(),
                    obj.jclass().name().as_usize()
                );
                interp.stack.push_jobj(obj);
                dispatch!(interp);
            } else {
                todo!(
                    "throw ClassNotFoundException {}",
                    target_class_name.as_str()
                );
            }
        }

        case_label_newarray!();
        {
            let interp = access_interpreter!();
            let array_type = ArrayType::from(interp.read_operand());
            let count = interp.stack.pop::<JInt>();
            if count < 0 {
                todo!("throw NegativeArraySizeException");
            }
            let preloaded_classes = interp.vm.preloaded_classes();
            let thread = Thread::current();
            let arr: ObjectPtr = match array_type {
                ArrayType::Boolean => {
                    JArray::new(count, preloaded_classes.bool_arr_cls(), thread).cast()
                }
                ArrayType::Char => {
                    JArray::new(count, preloaded_classes.char_arr_cls(), thread).cast()
                }
                ArrayType::Float => {
                    JArray::new(count, preloaded_classes.float_arr_cls(), thread).cast()
                }
                ArrayType::Double => {
                    JArray::new(count, preloaded_classes.double_arr_cls(), thread).cast()
                }
                ArrayType::Byte => {
                    JArray::new(count, preloaded_classes.byte_arr_cls(), thread).cast()
                }
                ArrayType::Short => {
                    JArray::new(count, preloaded_classes.short_arr_cls(), thread).cast()
                }
                ArrayType::Int => {
                    JArray::new(count, preloaded_classes.int_arr_cls(), thread).cast()
                }
                ArrayType::Long => {
                    JArray::new(count, preloaded_classes.long_arr_cls(), thread).cast()
                }
            };
            interp.stack.push_jobj(arr);
            dispatch!(interp);
        }

        case_label_nop!();
        {
            let interp = access_interpreter!();
            if interp.pc.is_not_null() {
                // unreachable
                log::trace!("{}", interp.stack.stack_trace_str());
                panic!();
            }
            dispatch!(interp);
        }

        case_label_pop!();
        {
            let interp = access_interpreter!();
            interp.stack.discard::<JInt>();
            dispatch!(interp);
        }

        case_label_pop2!();
        {
            let interp = access_interpreter!();
            interp.stack.discard::<JLong>();
            dispatch!(interp);
        }

        case_label_putfield!();
        {
            let interp = access_interpreter!();
            let index = u16::from(interp.read_operand());
            let index = (index << 8) | u16::from(interp.read_operand());
            let field_ref = interp
                .stack
                .frame()
                .class()
                .class_data()
                .cp
                .get_field_ref(index);
            if let Ok(target_class) = interp
                .vm
                .bootstrap_class_loader
                .load_class(field_ref.class_name.as_str())
            {
                let (target_field, _) = target_class.get_field(&field_ref);
                let field_class = match target_field.field_class(Thread::current()) {
                    Ok(field_class) => field_class,
                    Err(_) => todo!(),
                };
                log::trace!(
                    "prepare putfield, target {}.{} type {}, obj_ref: {}, field_offset: {}",
                    target_class.name().as_str(),
                    target_field.name().as_str(),
                    field_class.name().as_str(),
                    "prepare",
                    target_field.layout_offset()
                );
                let preloaded_classes = interp.vm.as_ref().preloaded_classes();
                if preloaded_classes.is_bool_cls(field_class)
                    || preloaded_classes.is_byte_cls(field_class)
                {
                    let value = interp.stack.pop::<JInt>() as JByte;
                    let obj_ref = interp.stack.pop_jobj();
                    target_field.set_typed_value(obj_ref, value);
                } else if preloaded_classes.is_char_cls(field_class)
                    || preloaded_classes.is_short_cls(field_class)
                {
                    let value = interp.stack.pop::<JInt>() as JShort;
                    let obj_ref = interp.stack.pop_jobj();
                    target_field.set_typed_value(obj_ref, value);
                } else if preloaded_classes.is_int_cls(field_class) {
                    let value = interp.stack.pop::<JInt>();
                    let obj_ref = interp.stack.pop_jobj();

                    log::trace!(
                        "prepare putfield int, class {}, obj 0x{:x}, field {}, field_offset: {}",
                        field_class.name().as_str(),
                        obj_ref.as_isize(),
                        target_field.name().as_str(),
                        target_field.layout_offset()
                    );
                    target_field.set_typed_value(obj_ref, value);
                } else if preloaded_classes.is_float_cls(field_class) {
                    let value = interp.stack.pop::<JFloat>();
                    let obj_ref = interp.stack.pop_jobj();
                    target_field.set_typed_value(obj_ref, value);
                } else if preloaded_classes.is_long_cls(field_class) {
                    let value = interp.stack.pop::<JLong>();
                    let obj_ref = interp.stack.pop_jobj();
                    target_field.set_typed_value(obj_ref, value);
                } else if preloaded_classes.is_double_cls(field_class) {
                    let value = interp.stack.pop::<JDouble>();
                    let obj_ref = interp.stack.pop_jobj();
                    target_field.set_typed_value(obj_ref, value);
                } else {
                    let value = interp.stack.pop_jobj().as_mut_raw_ptr();
                    let obj_ref = interp.stack.pop_jobj();
                    target_field.set_typed_value(obj_ref, value);
                    log::trace!(
                        "prepare putfield, target {}.{} type {}, obj_ref: 0x{:x}, val: 0x{:x?}, field_offset: {}",
                        target_class.name().as_str(),
                        target_field.name().as_str(),
                        field_class.name().as_str(),
                        obj_ref.as_isize(),
                        value,
                        target_field.layout_offset()
                    );
                }
                dispatch!(interp);
            } else {
                todo!("throw ClassNotFoundException");
            }
        }

        case_label_putstatic!();
        {
            let interp = access_interpreter!();
            let index = u16::from(interp.read_operand());
            let index = (index << 8) | u16::from(interp.read_operand());
            let field_ref = interp
                .stack
                .frame()
                .class()
                .class_data()
                .cp
                .get_field_ref(index);
            if let Ok(_target_class) = interp
                .vm
                .bootstrap_class_loader
                .load_class(field_ref.class_name.as_str())
            {
                let (target_field, decl_cls) = _target_class.get_field(&field_ref);
                if target_field.is_null() {
                    todo!("throw NoSuchFieldError");
                }
                match decl_cls.initialize(Thread::current()) {
                    Ok(_) => {}
                    Err(_) => todo!(),
                }
                if !target_field.is_static() {
                    todo!("throw IncompatibleClassChangeError");
                }
                let field_class = match target_field.field_class(Thread::current()) {
                    Ok(field_class) => field_class,
                    Err(_) => {
                        log::trace!(
                            "putstatic {}#{} load {} failed",
                            decl_cls.name().as_str(),
                            target_field.name().as_str(),
                            target_field.descriptor().as_str(),
                        );
                        todo!()
                    }
                };
                let preloaded_classes = interp.vm.preloaded_classes();
                if preloaded_classes.is_bool_cls(field_class)
                    || preloaded_classes.is_byte_cls(field_class)
                {
                    let value = interp.stack.pop::<JInt>() as JByte;
                    target_field.set_static_value(decl_cls, value);
                } else if preloaded_classes.is_char_cls(field_class)
                    || preloaded_classes.is_short_cls(field_class)
                {
                    let value = interp.stack.pop::<JInt>() as JShort;
                    target_field.set_static_value(decl_cls, value);
                } else if preloaded_classes.is_int_cls(field_class) {
                    let value = interp.stack.pop::<JInt>();
                    target_field.set_static_value(decl_cls, value);
                } else if preloaded_classes.is_float_cls(field_class) {
                    let value = interp.stack.pop::<JFloat>();
                    target_field.set_static_value(decl_cls, value);
                } else if preloaded_classes.is_long_cls(field_class) {
                    let value = interp.stack.pop::<JLong>();
                    target_field.set_static_value(decl_cls, value);
                } else if preloaded_classes.is_double_cls(field_class) {
                    let value = interp.stack.pop::<JDouble>();
                    target_field.set_static_value(decl_cls, value);
                } else {
                    let value = interp.stack.pop_jobj().as_mut_raw_ptr();
                    log::trace!(
                        "setstatic {}#{} : cls 0x{:x}  val {:x?} success, offset: {}",
                        decl_cls.name().as_str(),
                        target_field.name().as_str(),
                        decl_cls.as_isize(),
                        value,
                        target_field.layout_offset()
                    );
                    target_field.set_static_value(decl_cls, value);
                }
                dispatch!(interp);
            } else {
                todo!("throw ClassNotFoundException");
            }
        }

        case_label_ret!();
        {
            let interp = access_interpreter!();
            let index = interp.read_operand();
            interp.pc = interp.stack.load_jobj(isize::from(index)).as_address();
            dispatch!(interp);
        }

        case_label_array_load!(saload, JShortArrayPtr, JShort, JInt);
        case_label_array_store!(sastore, JShortArrayPtr, JShort, JInt);

        case_label_sipush!();
        {
            let interp = access_interpreter!();
            let val = JInt::from(interp.read_operand_i16());
            interp.stack.push(val);
            dispatch!(interp);
        }

        case_label_swap!();
        {
            let interp = access_interpreter!();
            interp.stack.swap();
            dispatch!(interp);
        }

        case_label_tableswitch!();
        {
            let interp = access_interpreter!();
            let op_addr = interp.pc.offset(-1);
            interp.skip_operands(3);
            let default_offset = interp.read_operand_i32();
            let low = interp.read_operand_i32();
            let high = interp.read_operand_i32();
            let index = interp.stack.pop::<JInt>();
            if index < low || index > high {
                interp.pc = op_addr.offset(Self::num2isize(default_offset) * 4);
            } else {
                let branch_offset = interp.peek_operand_as_int(Self::num2isize(index - low) * 4);
                interp.pc = op_addr.offset(Self::num2isize(branch_offset));
            }
            dispatch!(interp);
        }

        case_label_wide!();
        {
            let interp = access_interpreter!();
            let op_code: JvmInstruction = Self::op_code_as_instr(interp.read_operand());
            let index = interp.read_operand_u16();
            match op_code {
                JvmInstruction::ILoad => do_num_load!(JInt, index),
                JvmInstruction::FLoad => do_num_load!(JFloat, index),
                JvmInstruction::ALoad => {
                    let val = interp.stack.load_jobj(Self::num2isize(index));
                    interp.stack.push_jobj(val);
                }
                JvmInstruction::LLoad => do_num_load!(JLong, index),
                JvmInstruction::DLoad => do_num_load!(JDouble, index),
                JvmInstruction::IStore => do_num_store!(JInt, index),
                JvmInstruction::FStore => do_num_store!(JFloat, index),
                JvmInstruction::AStore => {
                    let val = interp.stack.pop_jobj();
                    interp.stack.store_jobj(val, Self::num2isize(index));
                }
                JvmInstruction::LStore => do_num_store!(JLong, index),
                JvmInstruction::DStore => do_num_store!(JDouble, index),
                JvmInstruction::Ret => {
                    interp.pc = interp.stack.load_jobj(Self::num2isize(index)).as_address()
                }
                JvmInstruction::IInc => {
                    let const_val = JInt::from(interp.read_operand_i16());
                    interp.stack.iinc(const_val, Self::num2isize(index));
                }
                _ => {
                    todo!("invalid classfile format");
                }
            }
            dispatch!(interp);
        }

        case_label_return!();
        {
            let interp = access_interpreter!();
            log::trace!(
                "restore_invoker_frame method {}#{}, 0x{:x}, locals {}",
                interp.stack.frame().class().name().as_str(),
                interp.stack.frame().method().name().as_str(),
                interp.stack.frame().method().as_isize(),
                interp.stack.frame().method().max_locals()
            );
            if interp.stack.is_top_java_frame() {
                interp.restore_invoker_frame();
                return JValue::with_int_val(0);
            }
            interp.restore_invoker_frame();
            dispatch!(interp);
        }

        case_label_impdep1!();
        {
            let interp = access_interpreter!();
            dispatch!(interp);
        }

        case_label_impdep2!();
        {
            let interp = access_interpreter!();
            dispatch!(interp);
        }

        case_label_breakpoint!();
        {
            let interp = access_interpreter!();
            dispatch!(interp);
        }
        return JValue::with_int_val(0);
    }

    fn create_dimension_array(
        &self,
        dimension_idx: u8,
        dimensions_end_idx: u8,
        dimensions_class_name: SymbolPtr,
        parent_dimension: JArrayPtr,
        dimension_current_class: JClassPtr,
    ) {
        //let dimension_it_class_name = &dimensions_class_name.as_str()[usize::from(dimension_idx)..];
        let parent_len = parent_dimension.length();
        let dimension_length = self
            .stack
            .peek_int(isize::from(dimensions_end_idx - dimension_idx));
        let thread = Thread::current();
        if dimension_idx < dimensions_end_idx {
            for parent_idx in 0..parent_len {
                let current_dimension_arr =
                    JArray::new(dimension_length, dimension_current_class, thread);
                parent_dimension.set(parent_idx, current_dimension_arr.cast());
                self.create_dimension_array(
                    dimension_idx + 1,
                    dimensions_end_idx,
                    dimensions_class_name,
                    current_dimension_arr,
                    dimension_current_class.class_data().component_type(),
                );
            }
        } else {
            for parent_idx in 0..parent_len {
                let current_dimension_arr =
                    JArray::new(dimension_length, dimension_current_class, thread);
                parent_dimension.set(parent_idx, current_dimension_arr.cast());
            }
        }
    }

    // #[inline(always)]
    fn invoke_method(
        &mut self,
        obj_ref: ObjectPtr,
        class: JClassPtr,
        method: MethodPtr,
        args_count: isize,
        args_slots: isize,
        obj_ref_size: isize,
        is_java_top: bool,
    ) {
        debug_assert!(args_count == method.params().length() as isize);
        // todo: synchronized

        let prev_pc = self.pc;
        self.pc = Address::new(method.code());
        if method.is_not_native() {
            self.stack.new_call_frame(
                class,
                method,
                args_slots,
                obj_ref_size,
                prev_pc,
                is_java_top,
                self.thread,
            );
        } else {
            self.stack.new_native_call_frame(
                class,
                method,
                args_slots,
                obj_ref_size,
                prev_pc,
                is_java_top,
                self.thread,
            );

            log::trace!(
                "call native method {}:{}, descriptor {}, code: {}",
                class.name().as_str(),
                method.name().as_str(),
                method.descriptor().as_str(),
                method.code().is_null()
            );
            let ret_type = method.ret_type();
            let ret_is_void = JClass::is_void(ret_type, self.vm);

            if method.native_fn().is_null() {
                todo!("throw Exception");
            }
            let ret_val = self.invoke_native_fn(class, method, obj_ref, obj_ref_size);

            self.restore_invoker_frame();

            if !ret_is_void {
                log::trace!("invoke_native_fn push value: 0x{:x}", ret_val.long_val());
                if JClass::is_long(ret_type, self.vm) || JClass::is_double(ret_type, self.vm) {
                    self.stack.push::<JLong>(ret_val.long_val());
                } else if ret_type.is_not_null() && JClass::is_primitive(ret_type) {
                    self.stack.push::<JInt>(ret_val.int_val());
                } else {
                    self.stack.push_jobj(ret_val.obj_val());
                }
            }
            return;
        }
        // Self::execute(self, class, method, is_root_frame);
    }

    fn invoke_native_fn(
        &self,
        class: JClassPtr,
        method: MethodPtr,
        objref: ObjectPtr,
        obj_ref_size: isize,
    ) -> JValue {
        debug_assert!(method.is_native());
        debug_assert!(!method.is_static() as isize == obj_ref_size);
        debug_assert!(method.native_fn().is_not_null());
        let params = method.params();
        let func = method.native_fn().raw_ptr() as usize;
        let vm = self.vm;
        let jni_env = vm.jni().get_env_handle();
        log::trace!("invoke_native_fn params_length: {}", params.length());
        let target_ref = if obj_ref_size == 0 {
            class.as_c_ptr()
        } else {
            objref.as_c_ptr()
        };
        let ret_val: JLong;
        match params.length() {
            0 => {
                #[cfg(all(target_arch = "x86_64", any(target_os = "linux", target_os = "macos")))]
                unsafe {
                    use std::arch::asm;
                    asm!(
                        "call {}",
                        in(reg) func,
                        in("rdi") jni_env,
                        in("rsi") target_ref,
                        out("rax") ret_val,
                        clobber_abi("C"),
                    );
                }
                #[cfg(all(target_arch = "aarch64", any(target_os = "linux", target_os = "macos")))]
                unsafe {
                    use std::arch::asm;
                    asm!(
                        "blr {}",
                        in(reg) func,
                        inout("x0") jni_env => ret_val,
                        in("x1") target_ref,
                        clobber_abi("C"),
                    );
                }
                #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
                unsafe {
                    use std::arch::asm;
                    asm!(
                        "call {}",
                        in(reg) func,
                        in("rcx") jni_env,
                        in("rdx") target_ref,
                        out("rax") ret_val,
                        clobber_abi("C"),
                    );
                }
            }
            1 => {
                let mut slot = 0;
                let arg0 = self.get_argument_as_jlong(vm, obj_ref_size, params, 0, &mut slot);

                #[cfg(all(target_arch = "x86_64", any(target_os = "linux", target_os = "macos")))]
                unsafe {
                    use std::arch::asm;
                    asm!(
                        "call {}",
                        in(reg) func,
                        in("rdi") jni_env,
                        in("rsi") target_ref,
                        in("rdx") arg0,
                        out("rax") ret_val,
                        clobber_abi("C"),
                    );
                }
                #[cfg(all(target_arch = "aarch64", any(target_os = "linux", target_os = "macos")))]
                unsafe {
                    use std::arch::asm;
                    asm!(
                        "blr {}",
                        in(reg) func,
                        inout("x0") jni_env => ret_val,
                        in("x1") target_ref,
                        in("x2") arg0,
                        clobber_abi("C"),
                    );
                }
                #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
                unsafe {
                    use std::arch::asm;
                    asm!(
                        "call {}",
                        in(reg) func,
                        in("rcx") jni_env,
                        in("rdx") target_ref,
                        in("r8") arg0,
                        out("rax") ret_val,
                        clobber_abi("C"),
                    );
                }
            }
            2 => {
                let mut slot = 0;
                let arg0 = self.get_argument_as_jlong(vm, obj_ref_size, params, 0, &mut slot);
                let arg1 = self.get_argument_as_jlong(vm, obj_ref_size, params, 1, &mut slot);

                #[cfg(all(target_arch = "x86_64", any(target_os = "linux", target_os = "macos")))]
                unsafe {
                    use std::arch::asm;
                    asm!(
                        "call {}",
                        in(reg) func,
                        in("rdi") jni_env,
                        in("rsi") target_ref,
                        in("rdx") arg0,
                        in("rcx") arg1,
                        out("rax") ret_val,
                        clobber_abi("C"),
                    );
                }
                #[cfg(all(target_arch = "aarch64", any(target_os = "linux", target_os = "macos")))]
                unsafe {
                    use std::arch::asm;
                    asm!(
                        "blr {}",
                        in(reg) func,
                        inout("x0") jni_env => ret_val,
                        in("x1") target_ref,
                        in("x2") arg0,
                        in("x3") arg1,
                        clobber_abi("C"),
                    );
                }
                #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
                unsafe {
                    use std::arch::asm;
                    asm!(
                        "call {}",
                        in(reg) func,
                        in("rcx") jni_env,
                        in("rdx") target_ref,
                        in("r8") arg0,
                        in("r9") arg1,
                        out("rax") ret_val,
                        clobber_abi("C"),
                    );
                }
            }
            3 => {
                let mut slot = 0;
                let arg0 = self.get_argument_as_jlong(vm, obj_ref_size, params, 0, &mut slot);
                let arg1 = self.get_argument_as_jlong(vm, obj_ref_size, params, 1, &mut slot);
                let arg2 = self.get_argument_as_jlong(vm, obj_ref_size, params, 2, &mut slot);

                #[cfg(all(target_arch = "x86_64", any(target_os = "linux", target_os = "macos")))]
                unsafe {
                    use std::arch::asm;
                    asm!(
                        "call {}",
                        in(reg) func,
                        in("rdi") jni_env,
                        in("rsi") target_ref,
                        in("rdx") arg0,
                        in("rcx") arg1,
                        in("r8") arg2,
                        out("rax") ret_val,
                        clobber_abi("C"),
                    );
                }
                #[cfg(all(target_arch = "aarch64", any(target_os = "linux", target_os = "macos")))]
                unsafe {
                    use std::arch::asm;
                    asm!(
                        "blr {}",
                        in(reg) func,
                        inout("x0") jni_env => ret_val,
                        in("x1") target_ref,
                        in("x2") arg0,
                        in("x3") arg1,
                        in("x4") arg2,
                        clobber_abi("C"),
                    );
                }
                #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
                unsafe {
                    use std::arch::asm;
                    asm!(
                        "mov [rsp+32], {arg2}",
                        "call {func}",
                        func = in(reg) func,
                        in("rcx") jni_env,
                        in("rdx") target_ref,
                        in("r8") arg0,
                        in("r9") arg1,
                        arg2 = in(reg) arg2,
                        out("rax") ret_val,
                        clobber_abi("C"),
                    );
                }
            }
            4 => {
                let mut slot = 0;
                let arg0 = self.get_argument_as_jlong(vm, obj_ref_size, params, 0, &mut slot);
                let arg1 = self.get_argument_as_jlong(vm, obj_ref_size, params, 1, &mut slot);
                let arg2 = self.get_argument_as_jlong(vm, obj_ref_size, params, 2, &mut slot);
                let arg3 = self.get_argument_as_jlong(vm, obj_ref_size, params, 3, &mut slot);

                #[cfg(all(target_arch = "x86_64", any(target_os = "linux", target_os = "macos")))]
                unsafe {
                    use std::arch::asm;
                    asm!(
                        "call {}",
                        in(reg) func,
                        in("rdi") jni_env,
                        in("rsi") target_ref,
                        in("rdx") arg0,
                        in("rcx") arg1,
                        in("r8") arg2,
                        in("r9") arg3,
                        out("rax") ret_val,
                        clobber_abi("C"),
                    );
                }
                #[cfg(all(target_arch = "aarch64", any(target_os = "linux", target_os = "macos")))]
                unsafe {
                    use std::arch::asm;
                    asm!(
                        "blr {}",
                        in(reg) func,
                        inout("x0") jni_env => ret_val,
                        in("x1") target_ref,
                        in("x2") arg0,
                        in("x3") arg1,
                        in("x4") arg2,
                        in("x5") arg3,
                        clobber_abi("C"),
                    );
                }
                #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
                unsafe {
                    use std::arch::asm;
                    asm!(
                        "mov [rsp+40], {arg3}",
                        "mov [rsp+32], {arg2}",
                        "call {func}",
                        func = in(reg) func,
                        in("rcx") jni_env,
                        in("rdx") target_ref,
                        in("r8") arg0,
                        in("r9") arg1,
                        arg2 = in(reg) arg2,
                        arg3 = in(reg) arg3,
                        out("rax") ret_val,
                        clobber_abi("C"),
                    );
                }
            }
            5 => {
                let mut slot = 0;
                let arg0 = self.get_argument_as_jlong(vm, obj_ref_size, params, 0, &mut slot);
                let arg1 = self.get_argument_as_jlong(vm, obj_ref_size, params, 1, &mut slot);
                let arg2 = self.get_argument_as_jlong(vm, obj_ref_size, params, 2, &mut slot);
                let arg3 = self.get_argument_as_jlong(vm, obj_ref_size, params, 3, &mut slot);
                let arg4 = self.get_argument_as_jlong(vm, obj_ref_size, params, 4, &mut slot);

                #[cfg(all(target_arch = "x86_64", any(target_os = "linux", target_os = "macos")))]
                unsafe {
                    use std::arch::asm;
                    asm!(
                        "mov [rsp], {arg4}",
                        "call {stub}",
                        stub = in(reg) func,
                        in("rdi") jni_env,
                        in("rsi") target_ref,
                        in("rdx") arg0,
                        in("rcx") arg1,
                        in("r8") arg2,
                        in("r9") arg3,
                        arg4 = in(reg) arg4,
                        out("rax") ret_val,
                        clobber_abi("C"),
                    );
                }
                #[cfg(all(target_arch = "aarch64", any(target_os = "linux", target_os = "macos")))]
                unsafe {
                    use std::arch::asm;
                    asm!(
                        "blr {}",
                        in(reg) func,
                        inout("x0") jni_env => ret_val,
                        in("x1") target_ref,
                        in("x2") arg0,
                        in("x3") arg1,
                        in("x4") arg2,
                        in("x5") arg3,
                        in("x6") arg4,
                        clobber_abi("C"),
                    );
                }
                #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
                unsafe {
                    use std::arch::asm;
                    asm!(
                        "mov [rsp+48], {arg4}",
                        "mov [rsp+40], {arg3}",
                        "mov [rsp+32], {arg2}",
                        "call {func}",
                        func = in(reg) func,
                        in("rcx") jni_env,
                        in("rdx") target_ref,
                        in("r8") arg0,
                        in("r9") arg1,
                        arg2 = in(reg) arg2,
                        arg3 = in(reg) arg3,
                        arg4 = in(reg) arg4,
                        out("rax") ret_val,
                        clobber_abi("C"),
                    );
                }
            }
            _ => todo!(),
        }
        return JValue::with_long_val(ret_val);
    }

    #[inline(always)]
    fn restore_invoker_frame(&mut self) {
        log::trace!(
            "restore_invoker_frame method {:x}, locals {}",
            self.stack.frame().method().as_isize(),
            self.stack.frame().method().max_locals()
        );
        self.stack.ret_call_frame(&mut self.pc);
        if self.stack.frame().is_not_null() {
            log::trace!(
                "restored_invoker_frame method at {}#{}, method addr {:x}, locals {}, trace {}",
                self.stack.frame().class().name().as_str(),
                self.stack.frame().method().name().as_str(),
                self.stack.frame().method().as_isize(),
                self.stack.frame().method().max_locals(),
                self.stack.stack_trace_str(),
            );
        } else {
            log::trace!("restore_invoker_frame root===");
        }
    }

    #[inline]
    fn get_argument_as_jlong(
        &self,
        vm: VMPtr,
        obj_ref_size: isize,
        params: JArrayPtr,
        param_idx: isize,
        slot: &mut isize,
    ) -> JLong {
        let current_slot = *slot;
        let param_cls: JClassPtr = params.get_with_isize(param_idx).cast();
        let mut arg = JValue::default();
        if JClass::is_long(param_cls, vm) || JClass::is_double(param_cls, vm) {
            arg.set_long_val(self.stack.load::<JLong>(obj_ref_size + current_slot));
            *slot += 2;
        } else if param_cls.class_data().is_primitive() {
            arg.set_int_val(self.stack.load::<JInt>(obj_ref_size + current_slot));
            *slot += 1;
        } else {
            let obj_val = self.stack.load_jobj(obj_ref_size + current_slot);
            arg.set_obj_val(obj_val);
            debug_assert!(obj_val.is_null() || vm.heap().heap_contains(arg.obj_val().as_address()));
            *slot += 1;
        }
        return arg.long_val();
    }

    #[inline(always)]
    fn op_val_cmp<T: StackPrimitiveValue + Copy + std::cmp::PartialOrd>(&mut self, nan_cmp: JInt) {
        let val2: T = self.stack.pop();
        let val1: T = self.stack.pop();
        if val1 > val2 {
            self.stack.push::<JInt>(1);
        } else if val1 == val2 {
            self.stack.push::<JInt>(0);
        } else if val1 < val2 {
            self.stack.push::<JInt>(-1);
        } else {
            self.stack.push::<JInt>(nan_cmp);
        }
        dispatch!(self);
    }

    #[inline(always)]
    fn op_val_load<T: StackPrimitiveValue + Copy>(&mut self, index: u8) {
        let val = self.stack.load(isize::from(index));
        self.stack.push::<T>(val);
    }

    fn op_ldc(interp: &mut Interpreter, index: u16) {
        let frame_class = interp.stack.frame().class();
        let constant_tag = frame_class.class_data().cp.get_tag(index);
        match constant_tag {
            ConstantTag::Integer => {
                interp
                    .stack
                    .push(frame_class.class_data().cp.get_int32(index));
            }
            ConstantTag::Float => {
                interp
                    .stack
                    .push(frame_class.class_data().cp.get_float(index));
            }
            ConstantTag::String => {
                let symbol = frame_class.class_data().cp.get_string(index);
                let jstr = interp
                    .vm
                    .string_table
                    .from_symbol(symbol, Thread::current());
                interp.stack.push_jobj(jstr.cast());
            }
            ConstantTag::Class => {
                let class_name = frame_class.class_data().cp.get_class_name(index);
                if let Ok(resolved_class) = interp
                    .vm
                    .bootstrap_class_loader
                    .load_class(class_name.as_str())
                {
                    interp.stack.push_jobj(resolved_class.cast());
                } else {
                    todo!("throw ClassNotFoundException");
                }
            }
            ConstantTag::MethodType | ConstantTag::MethodHandle => {
                todo!();
            }
            _ => {
                todo!("invalid constant tag");
            }
        }
    }

    #[inline(always)]
    fn goto(&mut self, base_op_addr: Address, branch: i16) {
        let target_addr = base_op_addr.offset(Self::num2isize(branch));
        let op_code = target_addr.deref_as_u8();
        self.pc = target_addr.offset(std::mem::size_of::<u8>() as isize);
        reserve_value!(self as *mut Self as usize);
        goto_label_addr!(OP_CODE_TABLE[usize::from(op_code)]);
    }

    #[inline(always)]
    fn goto_w(&mut self, base_op_addr: Address, branch: i32) {
        let target_addr = base_op_addr.offset(Self::num2isize(branch));
        let op_code = target_addr.deref_as_u8();
        self.pc = target_addr.offset(std::mem::size_of::<u8>() as isize);
        reserve_value!(self as *mut Self as usize);
        goto_label_addr!(OP_CODE_TABLE[usize::from(op_code)]);
    }

    #[inline(always)]
    fn read_operand(&mut self) -> u8 {
        let operand;
        unsafe { operand = *self.pc.raw_ptr() }
        self.pc = self.pc.offset(1);
        return operand;
    }

    #[inline(always)]
    fn read_op<V: Copy>(&mut self) -> V {
        let operand: V;
        unsafe { operand = *(self.pc.raw_ptr() as *const V) }
        self.pc = self.pc.offset(1);
        return operand;
    }

    #[inline(always)]
    fn read_operand_u16(&mut self) -> u16 {
        let val = u16::from(self.read_operand()) << 8;
        let val = val | u16::from(self.read_operand());
        return val;
    }

    #[inline(always)]
    fn read_operand_i16(&mut self) -> i16 {
        let val = i16::from(self.read_operand()) << 8;
        let val = val | i16::from(self.read_operand());
        return val;
    }

    #[inline(always)]
    fn read_operand_i32(&mut self) -> i32 {
        let val = (self.read_operand() as i32) << 24;
        let val = val | ((self.read_operand() as i32) << 16);
        let val = val | ((self.read_operand() as i32) << 8);
        let val = val | (self.read_operand() as i32);
        return val;
    }

    #[inline(always)]
    fn peek_operand_as_int(&self, offset: isize) -> i32 {
        let value = i32::from(self.pc.offset(offset).deref_as_u8()) << 24;
        let value = value | (i32::from(self.pc.offset(offset + 1).deref_as_u8()) << 16);
        let value = value | (i32::from(self.pc.offset(offset + 2).deref_as_u8()) << 8);
        let value = value | i32::from(self.pc.offset(offset + 3).deref_as_u8());
        return value;
    }

    #[inline(always)]
    fn skip_operands(&mut self, n: isize) {
        self.pc = self.pc.offset(n);
    }

    #[inline(always)]
    pub fn compute_args_slots(&self, method: MethodPtr, vm: VMPtr) -> isize {
        let params = method.params();
        let mut param_idx = params.length() - 1;

        let mut args_slots = 0;
        while param_idx >= 0 {
            let param: JClassPtr = params.get(param_idx).cast();
            if JClass::is_long(param, vm) || JClass::is_double(param, vm) {
                args_slots += 2;
            } else {
                args_slots += 1;
            }
            param_idx -= 1;
        }
        return args_slots;
    }

    #[inline(always)]
    fn num2isize<T>(num: T) -> isize
    where
        isize: TryFrom<T>,
    {
        match isize::try_from(num) {
            Ok(v) => return v,
            Err(_e) => unreachable!(),
        };
    }

    #[inline(always)]
    fn op_code_as_instr(op_code: u8) -> JvmInstruction {
        return unsafe { std::mem::transmute(op_code) };
    }
}

#[allow(dead_code)]
#[repr(u8)]
enum ArrayType {
    Boolean = 4,
    Char = 5,
    Float = 6,
    Double = 7,
    Byte = 8,
    Short = 9,
    Int = 10,
    Long = 11,
}

impl From<u8> for ArrayType {
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}
