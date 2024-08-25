use std::mem::size_of;

use crate::{
    handle::Handle,
    object::{
        array::{JArrayPtr, JByteArrayPtr, JCharArrayPtr},
        field::FieldPtr,
        method::MethodPtr,
        prelude::{JBoolean, JByte, JChar, JDouble, JFloat, JInt, JLong, JShort, Ptr},
        string::{HeapString, JString, JStringPtr, Utf16String},
        symbol::SymbolPtr,
        Object,
    },
    thread::ThreadPtr,
    value::JValue,
    vm::{VMError, VMPtr},
    JArray, JClassPtr, ObjectPtr,
};

macro_rules! make_java_lang_number_infos {
    ($(
        {$info_name:ident, $cls_name:expr, $val_ty:ty}
    ),*) => {
        $(
            #[derive(Default)]
            #[allow(unused)]
            pub(crate) struct $info_name {
                cls: JClassPtr,
                value: FieldPtr,
            }

            impl $info_name {
                pub(crate) fn new(cls: JClassPtr, thread: ThreadPtr) -> Result<Self, VMError> {
                    let vm = thread.vm();
                    let (value, _) = cls.get_field_with_name(vm.shared_objs().symbols().value);
                    assert!(value.is_not_null());
                    Ok(Self {
                        cls,
                        value,
                    })
                }

                #[allow(dead_code)]
                #[inline]
                pub(crate) fn get_value(&self, obj: ObjectPtr) -> $val_ty {
                    self.value.get_typed_value(obj)
                }

                #[allow(dead_code)]
                #[inline]
                pub(crate) fn cls(&self) -> JClassPtr {
                    self.cls
                }
            }
        )*
    };
}

make_java_lang_number_infos!(
    {JavaLangCharInfo, "java/lang/Char", JChar},
    {JavaLangByteInfo, "java/lang/Byte", JByte},
    {JavaLangBooleanInfo, "java/lang/Boolean", JBoolean},
    {JavaLangIntegerInfo, "java/lang/Integer", JInt},
    {JavaLangShortInfo, "java/lang/Short", JShort},
    {JavaLangLongInfo, "java/lang/Long", JLong},
    {JavaLangFloatInfo, "java/lang/Float", JFloat},
    {JavaLangDoubleInfo, "java/lang/Double", JDouble}
);

#[derive(Default)]
pub(crate) struct JavaLangStringInfo {
    jstring_cls: JClassPtr,
    value_field: FieldPtr,
}

impl JavaLangStringInfo {
    pub(crate) fn new(jstring_cls: JClassPtr, thread: ThreadPtr) -> Result<Self, VMError> {
        let value_field_name = thread.vm().symbol_table.get_or_insert("value");
        let (value_field, _) = jstring_cls.get_field_with_name(value_field_name);
        assert!(value_field.is_not_null());
        return Ok(Self {
            jstring_cls,
            value_field,
        });
    }

    pub fn create_with_utf8(&self, value: &str, thread: ThreadPtr) -> Handle<JString> {
        return self.create_with_utf16(&JString::str_to_utf16(value), thread);
    }

    pub fn create_with_utf16(&self, utf16_str: &Utf16String, thread: ThreadPtr) -> Handle<JString> {
        let vm = thread.vm();
        let utf16_len = utf16_str.len() as JInt;
        let char_arr: JCharArrayPtr = Handle::new(JArray::new(
            utf16_len,
            vm.preloaded_classes().char_arr_cls(),
            thread,
        ))
        .get_ptr()
        .cast();
        JString::char_arr_set_utf16_unchecked(char_arr, &utf16_str, utf16_len);
        let hash = HeapString::hash_utf16_str(utf16_str);
        let result = Handle::new(Object::new_with_hash(self.jstring_cls, thread, hash));
        self.value_field.set_typed_value(result.get_ptr(), char_arr);
        return result.cast();
    }

    pub fn create_permanent_with_utf16(
        &self,
        utf16_str: &Utf16String,
        thread: ThreadPtr,
    ) -> Handle<JString> {
        return self.create_permanent_with_utf16_hash(
            utf16_str,
            HeapString::hash_utf16_str(utf16_str),
            thread,
        );
    }

    pub fn create_permanent_with_utf16_hash(
        &self,
        utf16_str: &Utf16String,
        hash: JInt,
        thread: ThreadPtr,
    ) -> Handle<JString> {
        let vm = thread.vm();
        let utf16_len = utf16_str.len() as JInt;
        let char_arr: JCharArrayPtr = Handle::new(JArray::new(
            utf16_len,
            vm.preloaded_classes().char_arr_cls(),
            thread,
        ))
        .get_ptr()
        .cast();
        JString::char_arr_set_utf16_unchecked(char_arr, &utf16_str, utf16_len);
        let result = Handle::new(Object::new_permanent_with_hash(
            self.jstring_cls,
            thread,
            hash,
        ));
        self.value_field.set_typed_value(result.get_ptr(), char_arr);
        return result.cast();
    }

    pub fn create_string(&self, value: JCharArrayPtr, hash: JInt, thread: ThreadPtr) -> JStringPtr {
        let result = Object::new_with_hash(self.jstring_cls, thread, hash);
        self.value_field.set_typed_value(result, value);
        return result.cast();
    }

    pub fn create_permanent_with_chars(
        &self,
        value: JCharArrayPtr,
        hash: JInt,
        thread: ThreadPtr,
    ) -> JStringPtr {
        let result = Object::new_permanent_with_hash(self.jstring_cls, thread, hash);
        self.value_field.set_typed_value(result, value);
        // let count: JInt = value.length();
        // self.count_field.set_typed_value(result, count);
        return result.cast();
    }

    pub fn get_chars(&self, str: JStringPtr) -> JCharArrayPtr {
        return JCharArrayPtr::from_isize(self.value_field.fast_get_value(str.cast()) as isize);
    }
}

#[derive(Default)]
pub(crate) struct JavaLangThreadInfo {
    cls: JClassPtr,
    name: SymbolPtr,
    ctor: MethodPtr,
    daemon: FieldPtr,
    priority: FieldPtr,
    metadata_offset: u16,
}

impl JavaLangThreadInfo {
    pub(crate) fn new(cls: JClassPtr, thread: ThreadPtr) -> Result<Self, VMError> {
        let vm = thread.vm();
        let ctor_name = vm.shared_objs().symbols().ctor_init;
        let ctor_descriptor = vm.get_symbol("(Ljava/lang/ThreadGroup;Ljava/lang/Runnable;)V");
        let ctor = cls.resolve_local_method_unchecked(ctor_name, ctor_descriptor);
        let (daemon, _) = cls.get_field_with_name(vm.get_symbol("daemon"));
        let (priority, _) = cls.get_field_with_name(vm.get_symbol("priority"));
        debug_assert!(ctor.is_not_null());
        Ok(Self {
            cls,
            name: cls.name(),
            ctor,
            daemon,
            priority,
            metadata_offset: cls.class_data().metadata_offset(),
        })
    }

    pub(crate) fn new_jthread<F>(
        &self,
        thread_group: ObjectPtr,
        is_daemon: JBoolean,
        priority: JInt,
        created_action: F,
        thread: ThreadPtr,
    ) where
        F: FnOnce(Handle<Object>),
    {
        self.new_jthread_with_native_id(
            -1,
            thread_group,
            is_daemon,
            priority,
            created_action,
            thread,
        );
    }

    pub(crate) fn new_jthread_with_native_id<F>(
        &self,
        native_thread_id: JInt,
        thread_group: ObjectPtr,
        is_daemon: JBoolean,
        priority: JInt,
        created_action: F,
        thread: ThreadPtr,
    ) where
        F: FnOnce(Handle<Object>),
    {
        let jthread_handle = Handle::new(Object::new(self.cls, thread));
        let jthread = jthread_handle.as_ptr();
        self.daemon.set_typed_value(jthread, is_daemon);
        self.priority.set_typed_value(jthread, priority);
        self.set_native_thread_id(jthread, native_thread_id);
        created_action(jthread_handle);
        thread.vm().call_obj_void(
            jthread,
            self.ctor,
            &[
                JValue::with_obj_val(thread_group),
                JValue::with_obj_val(ObjectPtr::null()),
            ],
        );
    }

    pub(crate) fn name(&self) -> SymbolPtr {
        self.name
    }

    pub(crate) const fn metadata_size() -> u16 {
        return size_of::<JInt>() as u16;
    }

    fn native_thread_id(&self, obj: ObjectPtr) -> JInt {
        let native_thread_id: Ptr<JInt> = obj.read_value_ptr(self.metadata_offset as isize);
        return *native_thread_id;
    }

    fn set_native_thread_id(&self, obj: ObjectPtr, native_thread_id: JInt) {
        let mut field: Ptr<JInt> = obj.read_value_ptr(self.metadata_offset as isize);
        *field = native_thread_id;
    }
}

#[derive(Default)]
pub(crate) struct JavaLangThreadGroupInfo {
    cls: JClassPtr,
    ctor: MethodPtr,
}

impl JavaLangThreadGroupInfo {
    pub(crate) fn new(cls: JClassPtr, thread: ThreadPtr) -> Result<Self, VMError> {
        cls.initialize(thread)
            .map_err(|e| VMError::ClassInitError(e))?;
        let vm = thread.vm();
        let ctor_name = vm.shared_objs().symbols().ctor_init;
        let ctor_descriptor = vm.shared_objs().symbols().noargs_retv_descriptor;
        let ctor = cls.resolve_local_method_unchecked(ctor_name, ctor_descriptor);
        assert!(ctor.is_not_null());
        Ok(Self { cls, ctor })
    }

    pub fn new_permanent_thread_group(&self, thread: ThreadPtr) -> ObjectPtr {
        let thread_group = Object::new_permanent(self.cls, thread);
        thread.vm().call_obj_void(thread_group, self.ctor, &[]);
        return thread_group;
    }
}

#[derive(Default)]
pub(crate) struct JavaLangReflectFieldInfo {
    cls: JClassPtr,
    field_arr_cls: JClassPtr,
    ctor: MethodPtr,
    slot_field: FieldPtr,
}

impl JavaLangReflectFieldInfo {
    pub(crate) fn new(cls: JClassPtr, thread: ThreadPtr) -> Result<Self, VMError> {
        let vm = thread.vm();
        let field_arr_cls = vm
            .bootstrap_class_loader
            .load_class("[Ljava/lang/reflect/Field;")
            .map_err(|e| VMError::ClassLoaderErr(e))?;

        let method_name = vm.shared_objs().symbols().ctor_init;
        let method_descriptor = vm.get_symbol(
            "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/Class;IILjava/lang/String;[B)V",
        );
        let ctor = cls.resolve_local_method_unchecked(method_name, method_descriptor);
        let (slot_field, _) = cls.get_field_with_name(vm.shared_objs().symbols().slot);
        assert!(ctor.is_not_null());
        assert!(slot_field.is_not_null());
        return Ok(Self {
            cls,
            field_arr_cls,
            ctor,
            slot_field,
        });
    }

    pub(crate) fn cls(&self) -> JClassPtr {
        self.cls
    }

    pub(crate) fn field_arr_cls(&self) -> JClassPtr {
        self.field_arr_cls
    }

    pub(crate) fn constructor(&self) -> MethodPtr {
        self.ctor
    }

    pub(crate) fn slot_field(&self) -> FieldPtr {
        self.slot_field
    }
}

#[derive(Default)]
pub(crate) struct JavaLangReflectConstructorInfo {
    cls: JClassPtr,
    ctor_arr_cls: JClassPtr,
    clazz: FieldPtr,
    slot: FieldPtr,
    param_types: FieldPtr,
    modifiers: FieldPtr,
    ctor: MethodPtr,
}

impl JavaLangReflectConstructorInfo {
    pub(crate) fn new(cls: JClassPtr, thread: ThreadPtr) -> Result<Self, VMError> {
        let vm = thread.vm();
        let ctor_arr_cls = vm
            .bootstrap_class_loader
            .load_class("[Ljava/lang/reflect/Constructor;")
            .map_err(|e| VMError::ClassLoaderErr(e))?;
        let ctor = vm.shared_objs().symbols().ctor_init;
        let ctor_descriptor = vm.get_symbol(
            "(Ljava/lang/Class;[Ljava/lang/Class;[Ljava/lang/Class;IILjava/lang/String;[B[B)V",
        );
        let ctor = cls.resolve_local_method_unchecked(ctor, ctor_descriptor);
        let (clazz, _) = cls.get_field_with_name(vm.get_symbol("clazz"));
        let (slot, _) = cls.get_field_with_name(vm.shared_objs().symbols().slot);
        let (param_types, _) = cls.get_field_with_name(vm.get_symbol("parameterTypes"));
        let (modifiers, _) = cls.get_field_with_name(vm.get_symbol("modifiers"));

        assert!(ctor.is_not_null());
        assert!(clazz.is_not_null());
        assert!(slot.is_not_null());
        assert!(param_types.is_not_null());
        assert!(modifiers.is_not_null());
        return Ok(Self {
            cls,
            ctor_arr_cls,
            clazz,
            slot,
            param_types,
            modifiers,
            ctor,
        });
    }

    pub fn get_decl_cls(&self, ctor: ObjectPtr) -> JClassPtr {
        self.clazz.get_typed_value(ctor)
    }

    pub fn get_slot(&self, ctor: ObjectPtr) -> JInt {
        self.slot.get_typed_value(ctor)
    }

    pub fn get_param_types(&self, ctor: ObjectPtr) -> JArrayPtr {
        self.param_types.get_typed_value(ctor)
    }

    pub(crate) fn new_ctor(
        &self,
        decl_cls: JClassPtr,
        param_types_arr: JArrayPtr,
        checked_ex_arr: JArrayPtr,
        modifiers: JInt,
        slot: JInt,
        signature: JStringPtr,
        anno_arr: JByteArrayPtr,
        param_anno_arr: JByteArrayPtr,
        thread: ThreadPtr,
    ) -> Handle<Object> {
        let ctor_handle = Handle::new(Object::new(self.cls, thread));
        let ctor = ctor_handle.as_ptr();
        debug_assert_eq!(slot, JValue::with_int_val(slot).int_val());
        debug_assert_eq!(modifiers, JValue::with_int_val(modifiers).int_val());
        thread.vm().call_obj_void(
            ctor,
            self.ctor,
            &[
                JValue::with_obj_val(decl_cls.cast()),
                JValue::with_obj_val(param_types_arr.cast()),
                JValue::with_obj_val(checked_ex_arr.cast()),
                JValue::with_int_val(modifiers),
                JValue::with_int_val(slot),
                JValue::with_obj_val(signature.cast()),
                JValue::with_obj_val(anno_arr.cast()),
                JValue::with_obj_val(param_anno_arr.cast()),
            ],
        );
        return ctor_handle;
    }

    pub fn new_ctor_arr(&self, length: JInt, thread: ThreadPtr) -> Handle<JArray> {
        let arr_handle = Handle::new(JArray::new(length, self.ctor_arr_cls, thread));
        return arr_handle;
    }
}

#[derive(Default)]
pub(crate) struct JavaUtilPropertiesInfo {
    put_method: MethodPtr,
}

impl JavaUtilPropertiesInfo {
    pub(crate) fn new(cls: JClassPtr, thread: ThreadPtr) -> Result<Self, VMError> {
        let vm = thread.vm();
        let put_method = cls
            .resolve_class_method(
                vm.get_symbol("put"),
                vm.get_symbol("(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;"),
                vm,
            )
            .unwrap()
            .method;
        assert!(put_method.is_not_null());
        return Ok(Self { put_method });
    }

    pub fn put(&self, objref: ObjectPtr, key: ObjectPtr, value: ObjectPtr, vm: VMPtr) {
        vm.call_obj(
            objref,
            self.put_method,
            &[JValue::with_obj_val(key), JValue::with_obj_val(value)],
        );
    }
}

#[derive(Default)]
pub(crate) struct JavaSecurityPrivilegedActionInfo {
    cls: JClassPtr,
    run_name: SymbolPtr,
    run_descriptor: SymbolPtr,
}

impl JavaSecurityPrivilegedActionInfo {
    pub(crate) fn new(cls: JClassPtr, thread: ThreadPtr) -> Result<Self, VMError> {
        let run_name = thread.vm().get_symbol("run");
        let run_descriptor = thread.vm().get_symbol("()Ljava/lang/Object;");
        Ok(Self {
            cls,
            run_name,
            run_descriptor,
        })
    }

    pub(crate) fn cls(&self) -> JClassPtr {
        self.cls
    }

    pub(crate) fn run_name(&self) -> SymbolPtr {
        self.run_name
    }

    pub(crate) fn run_descriptor(&self) -> SymbolPtr {
        self.run_descriptor
    }
}

#[derive(Default)]
pub(crate) struct JavaIOFileInfo {
    #[allow(unused)]
    cls: JClassPtr,
    path: FieldPtr,
}

impl JavaIOFileInfo {
    pub(crate) fn new(cls: JClassPtr, thread: ThreadPtr) -> Result<Self, VMError> {
        let vm = thread.vm();
        let symbols = vm.shared_objs().symbols();
        let (path, _) = cls.get_field_with_name(symbols.path);
        return Ok(Self { cls, path });
    }

    pub fn get_path(&self, obj: ObjectPtr) -> JStringPtr {
        return self.path.get_typed_value(obj);
    }
}

#[derive(Default)]
pub(crate) struct JavaIOFileDescriptorInfo {
    cls: JClassPtr,
    #[cfg(target_family = "unix")]
    fd: FieldPtr,
    #[cfg(target_os = "windows")]
    handle: FieldPtr,
}

impl JavaIOFileDescriptorInfo {
    pub(crate) fn new(cls: JClassPtr, thread: ThreadPtr) -> Result<Self, VMError> {
        let vm = thread.vm();
        let symbols = vm.shared_objs().symbols();
        #[cfg(target_family = "unix")]
        let (fd, _) = cls.get_field_with_name(symbols.fd);
        #[cfg(target_os = "windows")]
        let (handle, _) = cls.get_field_with_name(symbols.handle);
        #[cfg(target_family = "unix")]
        assert!(fd.is_not_null());
        #[cfg(target_os = "windows")]
        assert!(handle.is_not_null());
        Ok(Self {
            cls,
            #[cfg(target_family = "unix")]
            fd,
            #[cfg(target_os = "windows")]
            handle,
        })
    }

    #[cfg(target_family = "unix")]
    pub(crate) fn get_fd(&self, obj_ref: ObjectPtr) -> JInt {
        debug_assert!(obj_ref.jclass() == self.cls);
        return self.fd.get_typed_value(obj_ref);
    }

    #[cfg(target_os = "windows")]
    pub(crate) fn get_handle(&self, obj_ref: ObjectPtr) -> JLong {
        debug_assert!(obj_ref.jclass() == self.cls);
        return self.handle.get_typed_value(obj_ref);
    }
}

#[derive(Default)]
pub(crate) struct JavaIOFileOutputStreamInfo {
    fd: FieldPtr,
}

impl JavaIOFileOutputStreamInfo {
    pub(crate) fn new(cls: JClassPtr, thread: ThreadPtr) -> Result<Self, VMError> {
        let vm = thread.vm();
        let symbols = vm.shared_objs().symbols();
        let (fd, _) = cls.get_field_with_name(symbols.fd);
        assert!(fd.is_not_null());
        Ok(Self { fd })
    }

    pub(crate) fn get_fd(&self, obj_ref: ObjectPtr) -> ObjectPtr {
        return self.fd.get_typed_value(obj_ref);
    }
}

#[derive(Default)]
pub struct JavaLangClassLoaderNativeLibraryInfo {
    handle: FieldPtr,
    from_class: FieldPtr,
}

impl JavaLangClassLoaderNativeLibraryInfo {
    pub(crate) fn new(cls: JClassPtr, thread: ThreadPtr) -> Result<Self, VMError> {
        let vm = thread.vm();
        let symbols = vm.shared_objs().symbols();
        let (handle, _) = cls.get_field_with_name(symbols.handle);
        let (from_class, _) = cls.get_field_with_name(symbols.from_class);
        assert!(handle.is_not_null());
        assert!(from_class.is_not_null());
        return Ok(Self { handle, from_class });
    }

    pub(crate) fn get_from_class(&self, obj: ObjectPtr) -> JClassPtr {
        return self.from_class.get_typed_value(obj);
    }

    pub(crate) fn set_handle(&self, obj: ObjectPtr, handle: JLong) {
        self.handle.set_typed_value(obj, handle);
    }
}
