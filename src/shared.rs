use crate::classfile::class_info::{
    JavaIOFileDescriptorInfo, JavaIOFileInfo, JavaIOFileOutputStreamInfo, JavaLangBooleanInfo,
    JavaLangByteInfo, JavaLangCharInfo, JavaLangClassLoaderNativeLibraryInfo, JavaLangDoubleInfo,
    JavaLangFloatInfo, JavaLangIntegerInfo, JavaLangLongInfo, JavaLangReflectConstructorInfo,
    JavaLangReflectFieldInfo, JavaLangShortInfo, JavaLangStringInfo, JavaLangThreadGroupInfo,
    JavaLangThreadInfo, JavaSecurityPrivilegedActionInfo, JavaUtilPropertiesInfo,
};
use crate::classfile::ClassLoadErr;
use crate::object::array::JArrayPtr;
use crate::object::class::{InitializationError, JClass, JClassPtr};
use crate::object::prelude::*;
use crate::thread::{Thread, ThreadPtr};
use crate::value::JValue;
use crate::vm::{VMError, VMPtr, VM};

use std::mem::size_of;

macro_rules! make_symbols {
    ($(
        {$symbol_name: ident, $symbol_val: expr}
    ),*) => {
        #[allow(non_snake_case)]
        #[derive(Default)]
        pub(crate) struct Symbols {
            $(
                pub(crate) $symbol_name: SymbolPtr,
            )*
        }

        impl Symbols {
            fn init(&mut self, vm: &VM) {
                $(
                    self.$symbol_name = vm.get_symbol($symbol_val);
                )*
            }
        }
    };
}

macro_rules! make_class_infos {
    ($(
        {$info_name:ident, $class_info:ident, $class_name_symbol:ident, [$($is_boxed_number:expr)?], [$($is_init:expr)?]}
    ),*) => {
        #[derive(Default)]
        pub(crate) struct ClassInfos {
            $(
                $info_name: $class_info,
            )*
        }

        impl ClassInfos {
            fn init(&mut self, vm: &VM, thread: ThreadPtr) -> Result<(), VMError> {
                $(
                    let $info_name = vm
                        .bootstrap_class_loader
                        .load_class_with_symbol(vm.shared_objs().symbols().$class_name_symbol)
                        .map_err(|e| VMError::ClassLoaderErr(e))?;

                    self.$info_name = $class_info::new($info_name, thread)?;
                )*
                $(
                    $(
                        $is_init;
                        $info_name.initialize(thread).map_err(|e| VMError::ClassInitError(e))?;
                    )*
                )*
                return Ok(());
            }

            pub(crate) fn get_unboxed_jnumber(&self, obj: ObjectPtr) -> Option<JValue> {
                let obj_cls = obj.jclass();
                $(
                    $(
                        $is_boxed_number;
                        if obj_cls == self.$info_name.cls() {
                            return Some(JValue::with_long_val(self.$info_name.get_value(obj) as JLong));
                        }
                    )*
                )*
                return None;
            }

            $(
                #[inline(always)]
                #[allow(unused)]
                pub(crate) fn $info_name(&self) -> &$class_info {
                    &self.$info_name
                }
            )*
        }
    };
}

macro_rules! preloaded_classes {
        ($(
            {$cls_field_name: ident, $rt_class_name: expr, $is_primitive: expr, $is_array: expr, $ele_size: expr, $component_type: ident, $is_class_fn: ident}
        ),*) => {
            pub(crate) struct PreloadedClasses {
                $($cls_field_name: JClassPtr,)*
                jclass_cls: JClassPtr,
                jobject_cls: JClassPtr,
                throwable_cls: JClassPtr,
                jclass_arr_cls: JClassPtr,
                jobject_arr_cls: JClassPtr,

                null: JClassPtr,
            }

            impl PreloadedClasses {
                pub fn new() -> Self {
                    return Self {
                        $($cls_field_name: JClassPtr::null(),)*
                        jclass_cls: JClassPtr::null(),
                        jobject_cls: JClassPtr::null(),
                        throwable_cls: JClassPtr::null(),
                        jclass_arr_cls: JClassPtr::null(),
                        jobject_arr_cls: JClassPtr::null(),
                        null: JClassPtr::null(),
                    };
                }

                pub fn is_preloaded(&self, cls: JClassPtr) -> bool {
                    $(
                        if self.$cls_field_name == cls {
                            return true;
                        }
                    )*
                    return false;
                }

                fn load_classes(&mut self, thread: ThreadPtr) {
                    $(
                        {
                            let class_name = thread.vm().symbol_table.get_or_insert($rt_class_name);
                            let $cls_field_name = JClass::new_system_class(class_name, $ele_size, $is_primitive, $is_array, self.$component_type, thread);

                            self.$cls_field_name = $cls_field_name;

                            log::trace!("load_classes cls addr {:x}, name: {}, name addr {:x}", self.$cls_field_name.as_usize(), self.$cls_field_name.name().as_str(), self.$cls_field_name.name().as_usize());
                        }
                    )*

                }

                fn setup(&mut self, class_cls: JClassPtr, thread: ThreadPtr) -> Result<(), InitializationError> {
                    $(
                        Object::init_header(self.$cls_field_name.cast(), class_cls);
                        self.$cls_field_name.initialize(thread)?;
                    )*

                    Object::init_header(self.jclass_arr_cls.cast(), class_cls);
                    Object::init_header(self.jobject_arr_cls.cast(), class_cls);
                    self.jclass_arr_cls.initialize(thread)?;
                    self.jobject_arr_cls.initialize(thread)?;
                    return Ok(());
                }

                fn add_to_class_loader(&self, thread: ThreadPtr) {
                    $(
                        if !$is_primitive {
                            thread.vm().bootstrap_class_loader.add_preloaded_class(self.$cls_field_name, thread);
                        }
                    )*
                }

                fn debug_verify(&self) {
                    $(
                        log::trace!("debug_verify {}, {:x} is_primitive {}", stringify!($cls_field_name),
                            self.$cls_field_name.as_usize(), JClass::is_primitive(self.$cls_field_name));
                        debug_assert_eq!(self.$cls_field_name.name().as_bytes(), $rt_class_name.as_bytes());
                    )*
                }

                pub fn get_primitive_class(&self, name: SymbolPtr) -> JClassPtr {
                    $(
                        if $is_primitive {
                            if self.$cls_field_name.name() == name {
                                return self.$cls_field_name;
                            }
                        }
                    )*
                    return JClassPtr::null();
                }

                $(
                    #[inline(always)]
                    pub fn $cls_field_name(&self) -> JClassPtr {
                        self.$cls_field_name
                    }

                    #[inline(always)]
                    pub fn $is_class_fn(&self, other: JClassPtr) -> bool {
                        debug_assert!(self.$cls_field_name.is_not_null());
                        return self.$cls_field_name == other;
                    }
                )*
            }
        };
    }

make_symbols!(
    {value, "value"},
    {slot, "slot"},
    {path, "path"},
    {handle, "handle"},
    {from_class, "fromClass"},
    {fd, "fd"},
    {fd_in, "in"},
    {fd_out, "out"},
    {fd_err, "err"},

    {ctor_init, "<init>"},
    {noargs_retv_descriptor, "()V"},

    {vm_str_cls_name, Symbols::VM_STR_CLS_NAME},
    {vm_cls_name, "rsvm/internal/Class"},
    {vm_cls_arr_name, "[rsvm/internal/Class;"},
    {vm_cp_cls_name, "rsvm/internal/ConstantPool"},

    {java_lang_Class, "java/lang/Class"},
    {java_lang_ClassLoader_NativeLibrary, "java/lang/ClassLoader$NativeLibrary"},
    {java_lang_String, "java/lang/String"},
    {java_lang_Thread, "java/lang/Thread"},
    {java_lang_ThreadGroup, "java/lang/ThreadGroup"},
    {java_util_Properties, "java/util/Properties"},
    {java_lang_reflect_Field, "java/lang/reflect/Field"},
    {java_lang_reflect_Constructor, "java/lang/reflect/Constructor"},
    {java_security_PrivilegedAction, "java/security/PrivilegedAction"},
    {java_io_File, "java/io/File"},
    {java_io_FileDescriptor, "java/io/FileDescriptor"},
    {java_io_FileOutputStream, "java/io/FileOutputStream"},
    {java_io_UnixFileSystem, "java/io/UnixFileSystem"},
    {java_io_WinNTFileSystem, "java/io/WinNTFileSystem"},

    {java_lang_Character, "java/lang/Character"},
    {java_lang_Byte, "java/lang/Byte"},
    {java_lang_Boolean, "java/lang/Boolean"},
    {java_lang_Integer, "java/lang/Integer"},
    {java_lang_Short, "java/lang/Short"},
    {java_lang_Long, "java/lang/Long"},
    {java_lang_Float, "java/lang/Float"},
    {java_lang_Double, "java/lang/Double"}
);

make_class_infos!(
    {java_lang_string_info, JavaLangStringInfo, java_lang_String, [], [true]},

    {java_lang_integer_info, JavaLangIntegerInfo, java_lang_Integer, [true], []},
    {java_lang_long_info, JavaLangLongInfo, java_lang_Long, [true], []},
    {java_lang_float_info, JavaLangFloatInfo, java_lang_Float, [true], []},
    {java_lang_double_info, JavaLangDoubleInfo, java_lang_Double, [true], []},
    {java_lang_boolean_info, JavaLangBooleanInfo, java_lang_Boolean, [], []},
    {java_lang_byte_info, JavaLangByteInfo, java_lang_Byte, [true], []},
    {java_lang_short_info, JavaLangShortInfo, java_lang_Short, [true], []},
    {java_lang_char_info, JavaLangCharInfo, java_lang_Character, [true], []},

    {java_lang_thread_info, JavaLangThreadInfo, java_lang_Thread, [], [true]},
    {java_lang_thread_group_info, JavaLangThreadGroupInfo, java_lang_ThreadGroup, [], [true]},
    {java_util_properties_info, JavaUtilPropertiesInfo, java_util_Properties, [], [true]},
    {java_lang_reflect_field_info, JavaLangReflectFieldInfo, java_lang_reflect_Field, [], [true]},
    {java_lang_reflect_constructor_info, JavaLangReflectConstructorInfo, java_lang_reflect_Constructor, [], [true]},
    {java_security_privileged_action_info, JavaSecurityPrivilegedActionInfo, java_security_PrivilegedAction, [], [true]},
    {java_io_file_info, JavaIOFileInfo, java_io_File, [], []},
    {java_io_file_descriptor_info, JavaIOFileDescriptorInfo, java_io_FileDescriptor, [], []},
    {java_io_file_output_stream_info, JavaIOFileOutputStreamInfo, java_io_FileOutputStream, [], []},

    {java_lang_classloader_native_library_info, JavaLangClassLoaderNativeLibraryInfo, java_lang_ClassLoader_NativeLibrary, [], [] }
);

preloaded_classes!(
    {char_cls, "char", true, false, size_of::<JChar>(), null, is_char_cls},
    {byte_cls, "byte", true, false, size_of::<JByte>(), null, is_byte_cls},
    {bool_cls, "boolean", true, false, size_of::<JBoolean>(), null, is_bool_cls},
    {int_cls, "int", true, false, size_of::<JInt>(), null, is_int_cls},
    {short_cls, "short", true, false, size_of::<JShort>(), null, is_short_cls},
    {long_cls, "long", true, false, size_of::<JLong>(), null, is_long_cls},
    {float_cls, "float", true, false, size_of::<JFloat>(), null, is_float_cls},
    {double_cls, "double", true, false, size_of::<JDouble>(), null, is_double_cls},

    {void_cls, "void", true, false, 0, void_cls, is_void_cls},

    {char_arr_cls, "[C", false, true, size_of::<JChar>(), char_cls, is_char_arr_cls},
    {byte_arr_cls, "[B", false, true, size_of::<JByte>(), byte_cls, is_byte_arr_cls},
    {bool_arr_cls, "[Z", false, true, size_of::<JBoolean>(), bool_cls, is_bool_arr_cls},
    {int_arr_cls, "[I", false, true, size_of::<JInt>(), int_cls, is_int_arr_cls},
    {short_arr_cls, "[S", false, true, size_of::<JShort>(), short_cls, is_short_arr_cls},
    {long_arr_cls, "[J", false, true, size_of::<JLong>(), long_cls, is_long_arr_cls},
    {float_arr_cls, "[F", false, true, size_of::<JFloat>(), float_cls, is_float_arr_cls},
    {double_arr_cls, "[D", false, true, size_of::<JDouble>(), double_cls, is_double_arr_cls}
);

impl PreloadedClasses {
    pub fn init(&mut self, vm_ptr: VMPtr, thread: ThreadPtr) -> Result<(), VMError> {
        let vm = vm_ptr.as_ref();

        self.jclass_cls = vm
            .bootstrap_class_loader
            .load_class("java/lang/Class")
            .map_err(|e| VMError::ClassLoaderErr(e))?;
        Object::init_header(self.jclass_cls.cast(), self.jclass_cls);

        self.jobject_cls = vm
            .bootstrap_class_loader
            .load_class("java/lang/Object")
            .map_err(|e| VMError::ClassLoaderErr(e))?;
        Object::init_header(self.jobject_cls.cast(), self.jclass_cls);

        self.jobject_arr_cls = vm
            .bootstrap_class_loader
            .load_class("[Ljava/lang/Object;")
            .map_err(|e| VMError::ClassLoaderErr(e))?;
        self.jclass_arr_cls = vm
            .bootstrap_class_loader
            .load_class("[Ljava/lang/Class;")
            .map_err(|e| VMError::ClassLoaderErr(e))?;

        log::trace!("jclass_cls {:x}", self.jclass_cls.as_usize());

        self.throwable_cls = vm
            .bootstrap_class_loader
            .load_class("java/lang/Throwable")
            .map_err(|e| VMError::ClassLoaderErr(e))?;

        self.setup(self.jclass_cls, thread)
            .map_err(|e| VMError::ClassInitError(e))?;

        self.jclass_cls
            .initialize(thread)
            .map_err(|e| VMError::ClassInitError(e))?;

        return Ok(());
    }

    fn bootstrap(&self, thread: ThreadPtr) {
        let mut self_ptr = Ptr::<Self>::from_ref(self);
        self_ptr.load_classes(thread);

        self.debug_verify();

        self.add_to_class_loader(thread);
    }

    pub fn jclass_cls(&self) -> JClassPtr {
        self.jclass_cls
    }

    pub fn jobject_cls(&self) -> JClassPtr {
        self.jobject_cls
    }

    pub fn jclass_arr_cls(&self) -> JClassPtr {
        self.jclass_arr_cls
    }

    pub fn jobject_arr_cls(&self) -> JClassPtr {
        self.jobject_arr_cls
    }
}

#[derive(Default)]
pub(crate) struct SharedObjects {
    symbols: Symbols,
    class_infos: ClassInfos,
    pub(crate) vm_str_cls: JClassPtr,
    pub(crate) empty_sys_arr: JArrayPtr,
    pub(crate) empty_jcls_arr: JArrayPtr,
    pub(crate) internal_arr_cls: JClassPtr,
    pub(crate) internal_cp_cls: JClassPtr,
    pub(crate) java_lang_thread_group: ObjectPtr,
    pub(crate) java_lang_cloneable_cls: JClassPtr,
    java_lang_class_inst_size: u16,
}

impl SharedObjects {
    pub(crate) fn init(&mut self, thread: ThreadPtr) {
        let vm = thread.vm();

        self.symbols.init_vm_str_cls_name(vm);
        let vm_str_cls_name = self.symbols.vm_str_cls_name;
        self.vm_str_cls =
            JClass::new_vm_internal_class(vm_str_cls_name, false, JClassPtr::null(), thread);
        Object::init_header_with_hash(
            vm_str_cls_name.cast(),
            self.vm_str_cls,
            vm_str_cls_name.hash_code(),
        );
        self.symbols.init(vm);
        assert!(self.vm_str_cls.name().as_str() == Symbols::VM_STR_CLS_NAME);

        let internal_cls = JClass::new_vm_internal_class(
            self.symbols.vm_cls_name,
            false,
            JClassPtr::null(),
            thread,
        );
        self.internal_arr_cls =
            JClass::new_vm_internal_class(self.symbols.vm_cls_arr_name, true, internal_cls, thread);
        self.internal_cp_cls = JClass::new_vm_internal_class(
            self.symbols.vm_cp_cls_name,
            false,
            JClassPtr::null(),
            thread,
        );

        self.empty_sys_arr = JArray::new_permanent(0, self.internal_arr_cls, thread);
    }

    pub(crate) fn post_init(&mut self, vm_ptr: VMPtr, thread: ThreadPtr) -> Result<(), VMError> {
        let vm = vm_ptr.as_ref();

        self.empty_jcls_arr = JArray::new(0, vm.preloaded_classes().jclass_arr_cls(), thread);

        self.class_infos.init(vm, thread)?;

        self.java_lang_cloneable_cls = vm
            .bootstrap_class_loader
            .load_class("java/lang/Cloneable")
            .map_err(|e| VMError::ClassLoaderErr(e))?;

        self.java_lang_thread_group = self
            .class_infos
            .java_lang_thread_group_info
            .new_permanent_thread_group(thread);

        Thread::create_jthread_and_bind(thread, self.java_lang_thread_group);

        debug_assert!(vm
            .preloaded_classes()
            .jobject_cls
            .class_data()
            .super_class()
            .is_null());
        debug_assert_eq!(
            vm.preloaded_classes().jclass_cls.class_data().super_class(),
            vm.preloaded_classes().jobject_cls
        );

        let jsystem_cls = vm
            .bootstrap_class_loader
            .load_class("java/lang/System")
            .map_err(|e| VMError::ClassLoaderErr(e))?;
        let jsystem_init_method = jsystem_cls
            .resolve_self_method(
                vm.get_symbol("initializeSystemClass"),
                self.symbols.noargs_retv_descriptor,
            )
            .map_err(|_e| {
                VMError::ClassLoaderErr(ClassLoadErr::InvalidFormat(
                    "No such method initializeSystemClass available".to_string(),
                ))
            })?;
        let sys_init_at = std::time::SystemTime::now();
        vm.call_static_void(jsystem_cls, jsystem_init_method.method, &[]);
        log::info!(
            "initializeSystemClass end elapsed {:#?} seconds",
            sys_init_at.elapsed().unwrap().as_secs()
        );
        return Ok(());
    }

    pub(crate) fn bootstrap(&self, java_lang_class_inst_size: u16) {
        log::trace!("java_lang_class_inst_size {}", java_lang_class_inst_size);
        let mut self_ptr = Ptr::<Self>::from_ref(self);
        assert!(self_ptr.is_bootstrapping());
        let thread = Thread::current();
        let vm = thread.vm();
        self_ptr.java_lang_class_inst_size = java_lang_class_inst_size;
        vm.preloaded_classes().bootstrap(thread);
    }

    pub(crate) fn resize_for_metadata(&self, class_name: SymbolPtr, inst_size: u16) -> u16 {
        if class_name == self.class_infos.java_lang_thread_info.name() {
            return inst_size + JavaLangThreadInfo::metadata_size();
        }
        return inst_size;
    }

    pub(crate) fn is_bootstrapping(&self) -> bool {
        return self.java_lang_class_inst_size == 0;
    }

    #[inline(always)]
    pub(crate) fn symbols(&self) -> &Symbols {
        &self.symbols
    }

    #[inline(always)]
    pub(crate) fn class_infos(&self) -> &ClassInfos {
        &self.class_infos
    }

    pub(crate) fn java_lang_class_inst_size(&self) -> u16 {
        debug_assert_ne!(self.java_lang_class_inst_size, 0);
        self.java_lang_class_inst_size
    }
}

impl Symbols {
    const VM_STR_CLS_NAME: &'static str = "rsvm/internal/VMString";

    fn init_vm_str_cls_name(&mut self, vm: &VM) {
        self.vm_str_cls_name = vm.get_symbol(Symbols::VM_STR_CLS_NAME);
    }
}
