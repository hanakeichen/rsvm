use jni::{
    objects::{JClass, JObject, JObjectArray, JString},
    sys::{jarray, jboolean, jbyteArray, jclass, jint, jobject, jstring},
    JNIEnv,
};

use crate::{
    native::jni::JNIEnvWrapper,
    object::{
        array::{JArrayPtr, JByteArrayPtr},
        field::FieldPtr,
        method::MethodPtr,
        prelude::JInt,
        string::JStringPtr,
    },
    thread::Thread,
    JArray, JClassPtr,
};

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_registerNatives<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
) {
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_forName0<'local>(
    env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
    name: JString<'local>,
    initialize: jboolean,
    _loader: JObject<'local>,
    _caller: JObject<'local>,
) -> jclass {
    type InternalJString = crate::object::string::JString;

    let vm = JNIEnvWrapper::from_raw_env(env.get_raw()).vm();
    let name = JStringPtr::from_raw(name.as_raw() as _);
    let name = InternalJString::to_rust_string(name, vm.as_ref());
    match vm.bootstrap_class_loader.load_binary_name_class(&name) {
        Ok(cls) => {
            if initialize == 1 {
                if let Err(_e) = cls.initialize(Thread::current()) {
                    todo!();
                }
            }
            cls.as_raw_ptr() as _
        }
        Err(_e) => todo!(),
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_isInstance<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
    _obj: JObject<'local>,
) -> jboolean {
    todo!();
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_isAssignableFrom<'local>(
    env: JNIEnv<'local>,
    obj_ref: JObject<'local>,
    cls: JClass<'local>,
) -> jboolean {
    if obj_ref.is_null() {
        todo!("throw NullPointerException");
    }
    let vm = JNIEnvWrapper::from_raw_env(env.get_raw()).vm();
    let obj_ref = JClassPtr::from_raw(obj_ref.as_raw() as _);
    let cls = JClassPtr::from_raw(cls.as_raw() as _);
    return if obj_ref.is_assignable_from(cls, vm) {
        1
    } else {
        0
    };
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_isInterface<'local>(
    _env: JNIEnv<'local>,
    obj_ref: JObject<'local>,
) -> jboolean {
    if obj_ref.is_null() {
        todo!("throw NullPointerException");
    }
    JClassPtr::from_raw(obj_ref.as_raw() as _)
        .class_data()
        .is_interface() as _
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_isArray<'local>(
    _env: JNIEnv<'local>,
    obj_ref: JObject<'local>,
) -> jboolean {
    if obj_ref.is_null() {
        todo!("throw NullPointerException");
    }
    JClassPtr::from_raw(obj_ref.as_raw() as _)
        .class_data()
        .is_array() as _
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_isPrimitive<'local>(
    _env: JNIEnv<'local>,
    obj_ref: JObject<'local>,
) -> jboolean {
    if obj_ref.is_null() {
        todo!("throw NullPointerException");
    }
    let obj_ref = JClassPtr::from_raw(obj_ref.as_raw() as _);
    return if obj_ref.class_data().is_primitive() {
        1
    } else {
        0
    };
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_getName0<'local>(
    env: JNIEnv<'local>,
    obj_ref: JObject<'local>,
) -> jstring {
    if obj_ref.is_null() {
        todo!("throw NullPointerException");
    }
    let wrapper = JNIEnvWrapper::from_raw_env(env.get_raw());
    let vm = wrapper.vm();
    let obj_ref = JClassPtr::from_raw(obj_ref.as_raw() as _);

    return vm
        .get_jstr_from_symbol(obj_ref.name(), Thread::current())
        .as_raw_ptr() as _;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_getClassLoader0<'local>(
    _env: JNIEnv<'local>,
    obj_ref: JObject<'local>,
) -> jobject {
    let obj_ref = JClassPtr::from_raw(obj_ref.as_raw() as _);
    return obj_ref.class_loader().as_raw_ptr() as jobject;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_getSuperclass<'local>(
    _env: JNIEnv<'local>,
    obj_ref: JObject<'local>,
) -> jclass {
    if obj_ref.is_null() {
        todo!("throw NullPointerException");
    }
    let obj_ref = JClassPtr::from_raw(obj_ref.as_raw() as _);
    return obj_ref.class_data().super_class().as_raw_ptr() as _;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_getInterfaces<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
) -> jarray {
    todo!();
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_getComponentType<'local>(
    _env: JNIEnv<'local>,
    obj_ref: JObject<'local>,
) -> jclass {
    if obj_ref.is_null() {
        todo!("throw NullPointerException");
    }
    let obj_ref = JClassPtr::from_raw(obj_ref.as_raw() as _);
    return obj_ref.class_data().component_type().as_raw_ptr() as _;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_getModifiers<'local>(
    _env: JNIEnv<'local>,
    obj_ref: JObject<'local>,
) -> jint {
    if obj_ref.is_null() {
        todo!("throw NullPointerException");
    }
    return JClassPtr::from_raw(obj_ref.as_raw() as _)
        .class_data()
        .access_flags() as jint;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_getSigners<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
) -> jarray {
    todo!();
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_setSigners<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
    _signers: JObjectArray<'local>,
) {
    todo!();
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_getEnclosingMethods<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
) -> jarray {
    todo!();
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_getDeclaringClass<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
) -> jclass {
    todo!();
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_getProtectionDomain0<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
) -> jobject {
    todo!();
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_setProtectionDomain0<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
    _pd: JObject<'local>,
) {
    todo!();
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_getPrimitiveClass<'local>(
    env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
    name: JString<'local>,
) -> jclass {
    type InternalJString = crate::object::string::JString;
    type InternalJClass = crate::object::class::JClass;

    let wrapper = JNIEnvWrapper::from_raw_env(env.get_raw());
    let vm = wrapper.vm();
    let jstr = JStringPtr::from_raw(name.as_raw() as *mut InternalJString);
    let symbol = vm.get_symbol_with_jstr(jstr);
    if symbol.is_null() {
        todo!(
            "throws ClassNotFoundException, jstr addr 0x{:x}",
            jstr.as_isize()
        );
    }
    let result = InternalJClass::get_primitive_class(symbol, vm);
    if result.is_null() {
        todo!("throw ClassNotFoundException");
    }
    log::trace!(
        "Java_java_lang_Class_getPrimitiveClass 0x{:x}",
        result.as_isize()
    );
    return result.as_raw_ptr() as jclass;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_getGenericSignature<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
) -> jstring {
    todo!();
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_getRawAnnotations<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
) -> jbyteArray {
    todo!();
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_getConstantPool<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
) -> jobject {
    todo!();
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_getDeclaredFields0<'local>(
    env: JNIEnv<'local>,
    obj_ref: JObject<'local>,
    _public_only: jboolean,
) -> jarray {
    type RObject = crate::object::Object;
    type RJValue = crate::value::JValue;

    let obj_ref: JClassPtr = JClassPtr::from_raw(obj_ref.as_raw() as _);
    let fields = obj_ref.class_data().fields();
    let thread = Thread::current();
    let vm = JNIEnvWrapper::from_raw_env(env.get_raw()).vm();
    let field_info = vm
        .shared_objs()
        .class_infos()
        .java_lang_reflect_field_info();
    let result = JArray::new(fields.length(), field_info.field_arr_cls(), thread);
    let field_cls = field_info.cls();
    let init_method = field_info.constructor();
    for idx in 0..fields.length() {
        let field: FieldPtr = fields.get(idx).cast();
        let j_field = RObject::new(field_cls, thread);

        let field_name = vm.get_jstr_from_symbol(field.name(), thread);
        let field_sig = JStringPtr::null(); // TODO
        let slot = {
            let mut slot = RJValue::with_long_val(0);
            slot.set_ushort_val(field.layout_offset());
            slot
        };

        log::trace!(
            "Java_java_lang_Class_getDeclaredFields0 {}#{}, original offset: {}, offset: {}",
            obj_ref.name().as_str(),
            field.name().as_str(),
            field.layout_offset(),
            slot.int_val()
        );

        vm.call_obj_void(
            j_field,
            init_method,
            &[
                RJValue::with_obj_val(obj_ref.cast()),
                RJValue::with_obj_val(field_name.cast()),
                RJValue::with_obj_val(field.field_class(thread).unwrap().cast()),
                RJValue::with_ushort_val(field.access_flags()),
                slot,
                RJValue::with_obj_val(field_sig.cast()),
                RJValue::with_obj_null(),
            ],
        );

        result.set(idx, j_field);
    }
    return result.as_raw_ptr() as jarray;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_getDeclaredMethods0<'local>(
    _env: JNIEnv<'local>,
    _obj_ref: JObject<'local>,
    _public_only: jboolean,
) -> jarray {
    todo!();
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_getDeclaredConstructors0<'local>(
    env: JNIEnv<'local>,
    obj_ref: JObject<'local>,
    public_only: jboolean,
) -> jarray {
    if obj_ref.is_null() {
        todo!("throw NullPointerException");
    }
    let obj_ref = JClassPtr::from_raw(obj_ref.as_raw() as _);
    let vm = JNIEnvWrapper::from_raw_env(env.get_raw()).vm();
    let methods = obj_ref.class_data().methods();
    let mut filtered_methods = Vec::new();
    let filtered_name = vm.shared_objs().symbols().ctor_init;
    let reflect_ctor_info = vm
        .shared_objs()
        .class_infos()
        .java_lang_reflect_constructor_info();
    let thread = Thread::current();

    for idx in 0..methods.length() {
        let method: MethodPtr = methods.get(idx).cast();
        if method.name() != filtered_name {
            continue;
        }
        if public_only == 1 && method.is_not_public() {
            continue;
        }
        let param_types_arr = {
            let method_params = method.params();
            let method_params_len = method_params.length();
            if method_params_len > 0 {
                let param_types_arr = JArray::new(
                    method_params_len,
                    vm.preloaded_classes().jclass_arr_cls(),
                    thread,
                );
                for idx in 0..method_params_len {
                    param_types_arr.set(idx, method_params.get(idx));
                }
                param_types_arr
            } else {
                vm.shared_objs().empty_jcls_arr
            }
        };
        let signature = JStringPtr::null(); // TODO
        let anno_arr = JByteArrayPtr::null(); // TODO
        let param_anno_arr = JByteArrayPtr::null(); // TODO
        let ctor = reflect_ctor_info.new_ctor(
            method.decl_cls(),
            param_types_arr,
            JArrayPtr::null(),
            method.access_flags() as JInt,
            idx,
            signature,
            anno_arr,
            param_anno_arr,
            thread,
        );
        filtered_methods.push(ctor);
    }
    let filtered_length = filtered_methods.len() as JInt;
    let result_arr = reflect_ctor_info.new_ctor_arr(filtered_length, thread);
    for idx in 0..filtered_length {
        result_arr.set(
            idx,
            unsafe { filtered_methods.get_unchecked(idx as usize) }.as_ptr(),
        );
    }

    return result_arr.as_ptr().as_raw_ptr() as _;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_getDeclaredClasses0<'local>(
    _env: JNIEnv<'local>,
    obj_ref: JObject<'local>,
) -> jarray {
    if obj_ref.is_null() {
        todo!("throw NullPointerException");
    }
    let _obj_ref = JClassPtr::from_raw(obj_ref.as_raw() as _);
    todo!();
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn Java_java_lang_Class_desiredAssertionStatus0<'local>(
    _env: JNIEnv<'local>,
    _cls_ref: JClass<'local>,
    _clazz: JClass<'local>,
) -> jboolean {
    return false as jboolean;
}
