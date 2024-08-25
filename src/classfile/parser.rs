use super::reader::ClassReader;
use super::ClassLoadErr;
use crate::classfile::descriptor::{Descriptor, DescriptorParser};
use crate::handle::Handle;
use crate::memory::heap::Heap;
use crate::object::array::JArray;
use crate::object::class::{FieldLayout, JClass, JClassPtr, VTable, VTableInfo};
use crate::object::constant_pool::{ConstantPool, ConstantTag};
use crate::object::field::{Field, FieldAccessFlags};
use crate::object::method::{ExceptionTable, Method, MethodPtr};
use crate::object::prelude::*;
use crate::thread::Thread;
use crate::vm::VM;
use std::convert::TryInto;
use std::f64;

const CLASS_FILE_MAGIC: u32 = 0xCAFEBABE;

pub struct ClassParser<'a> {
    jclass_loader: ObjectPtr,
    reader: Box<dyn ClassReader>,
    vm: &'a VM,
    this_class_name: SymbolPtr,
}

impl<'a> ClassParser<'a> {
    pub fn new(jclass_loader: ObjectPtr, reader: Box<dyn ClassReader>, vm: &'a VM) -> Self {
        ClassParser {
            jclass_loader,
            reader,
            vm,
            this_class_name: SymbolPtr::null(),
        }
    }

    pub fn parse_class(&mut self) -> Result<JClassPtr, ClassLoadErr> {
        let magic = self.reader.read_ubyte4()?;
        if magic != CLASS_FILE_MAGIC {
            return Err(ClassLoadErr::InvalidFormat(
                "cannot identify the magic number".to_string(),
            ));
        }
        let _minor_version = self.reader.read_ubyte2()?;
        let major_version = self.reader.read_ubyte2()?;
        if !Self::major_version_is_support(major_version) {
            return Err(ClassLoadErr::InvalidFormat(
                "unsupported class file version".to_string(),
            ));
        }
        let cp = self.parse_constant_pool()?;
        let access_flags = self.reader.read_ubyte2()?;
        let this_class = self.reader.read_ubyte2()?;
        let class_name = cp.get_class_name(this_class);
        self.this_class_name = class_name;
        let super_class_index = self.reader.read_ubyte2()?;
        let super_class_name = if super_class_index != 0 {
            let super_class_name = cp.get_class_name(super_class_index);
            super_class_name
        } else {
            Ptr::null()
        };

        let java_lang_class_bootstrapping = self.vm.shared_objs().symbols().java_lang_Class
            == class_name
            && self.vm.shared_objs().is_bootstrapping();
        let interfaces = self.parse_interfaces(&cp, java_lang_class_bootstrapping)?;
        let ParsedFields {
            fields,
            static_fields_size,
            inst_size,
            metadata_offset,
        } = self.parse_fields(&cp)?;
        if java_lang_class_bootstrapping {
            // bootstrapping
            self.vm.shared_objs().bootstrap(inst_size);
        }
        let mut init_method = MethodPtr::null();
        let methods = self.parse_methods(&cp, &mut init_method)?;

        let super_class = if super_class_name.is_not_null() {
            self.vm
                .bootstrap_class_loader
                .load_class(super_class_name.as_str())?
        } else {
            JClassPtr::null()
        };

        if java_lang_class_bootstrapping {
            assert_eq!("java/lang/Object", super_class.name().as_str());
            for iface_idx in 0..interfaces.length() {
                let iface_name: SymbolPtr = interfaces.get(iface_idx).cast();
                let iface = self
                    .vm
                    .bootstrap_class_loader
                    .load_class(iface_name.as_str())?;
                interfaces.set(iface_idx, iface.cast());
            }
        }

        let vtab_info = if ClassAccessFlags::is_interface(access_flags) {
            VTableInfo::default()
        } else {
            log::trace!(
                "compute_vtab_len for class {}, ifaces: addr {:x}, methods len {}",
                class_name.as_str(),
                interfaces.as_ptr().as_usize(),
                methods.length(),
            );
            VTable::obtain_vtab_info(
                access_flags,
                methods.as_ptr(),
                super_class,
                interfaces.as_ptr(),
                self.vm.shared_objs().symbols().ctor_init,
            )
        };

        log::trace!(
            "JClass::new_permanent parsed: {}, inst size {}, metadata_offset {}",
            class_name.as_str(),
            inst_size,
            metadata_offset,
        );
        let class = JClass::new_permanent(
            cp.as_ptr(),
            access_flags,
            class_name,
            super_class.cast(),
            interfaces.as_ptr(),
            fields.as_ptr(),
            methods.as_ptr(),
            static_fields_size,
            &vtab_info,
            inst_size,
            metadata_offset,
            self.jclass_loader,
            init_method,
            JClassPtr::null(),
            self.vm.preloaded_classes().jclass_cls(),
            Thread::current(),
        );
        debug_assert_eq!(class.class_data().methods().length(), methods.length());
        self.parse_class_attrs(&cp, class)?;
        debug_assert_eq!(self.reader.available_bytes().len(), 0);
        return Ok(class);
    }

    /// jvms-4.4
    fn parse_constant_pool(&mut self) -> Result<Handle<ConstantPool>, ClassLoadErr> {
        let cp_count = self.reader.read_ubyte2()?;
        // The constant_pool table is indexed from 1 to constant_pool_count - 1.
        let mut cp = Handle::new(ConstantPool::new(cp_count, Thread::current()));
        let mut index = 1;
        while index <= cp_count - 1 {
            let tag_u8: u8 = self.reader.read_ubyte1()?;
            let tag = ConstantTag::from(tag_u8);
            match tag {
                ConstantTag::Utf8 => {
                    let length = usize::from(self.reader.read_ubyte2()?);
                    let bytes = self.reader.peek_nbytes(length)?;
                    let symbol = std::str::from_utf8(bytes).map_err(|e| {
                        ClassLoadErr::InvalidFormat(format!("invalid CONSTANT_Utf8: {}", e))
                    })?;
                    cp.set_utf8(index, self.vm.symbol_table.get_or_insert(symbol));
                    self.reader.skip(length);
                }
                ConstantTag::Integer => cp.set_int32(index, self.reader.read_ubyte4()? as i32),
                ConstantTag::Float => cp.set_float(index, self.reader.read_ubyte4()? as f32),
                ConstantTag::Long => {
                    let high_bytes = (self.reader.read_ubyte4()? as JLong) << 32;
                    let low_bytes = self.reader.read_ubyte4()? as JLong;
                    cp.set_long(index, high_bytes | low_bytes);
                    index += 1
                }
                ConstantTag::Double => {
                    let high_bytes = (self.reader.read_ubyte4()? as u64) << 32;
                    let low_bytes = self.reader.read_ubyte4()?;
                    let bits = high_bytes | low_bytes as u64;
                    let result: JDouble = match bits {
                        0x7ff0000000000000 => f64::INFINITY,
                        0xfff0000000000000 => f64::NEG_INFINITY,
                        _bits @ 0x7ff0000000000001..=0x7fffffffffffffff
                        | _bits @ 0xfff0000000000001..=0xffffffffffffffff => f64::NAN,
                        _ => {
                            let s: i32 = if (bits >> 63) == 0 { 1 } else { -1 };
                            let e: u32 = ((bits >> 52) & 0x7ff) as u32;
                            let m: i64 = if e == 0 {
                                ((bits & 0xfffffffffffff) << 1) as i64
                            } else {
                                ((bits & 0xfffffffffffff) | 0x10000000000000) as i64
                            };
                            s as f64 * m as f64 * i32::pow(2, e - 1075) as f64
                        }
                    };
                    cp.set_double(index, result);
                    index += 1
                }
                ConstantTag::Class => cp.set_class_index(index, self.reader.read_ubyte2()?),
                ConstantTag::String => cp.set_string(index, self.reader.read_ubyte2()?),
                ConstantTag::Fieldref => cp.set_field_ref(
                    index,
                    self.reader.read_ubyte2()?,
                    self.reader.read_ubyte2()?,
                ),
                ConstantTag::Methodref => cp.set_method_ref(
                    index,
                    self.reader.read_ubyte2()?,
                    self.reader.read_ubyte2()?,
                ),
                ConstantTag::InterfaceMethodref => cp.set_interface_method_ref(
                    index,
                    self.reader.read_ubyte2()?,
                    self.reader.read_ubyte2()?,
                ),
                ConstantTag::NameAndType => cp.set_name_and_type(
                    index,
                    self.reader.read_ubyte2()?,
                    self.reader.read_ubyte2()?,
                ),
                ConstantTag::MethodHandle => cp.set_method_handle(
                    index,
                    self.reader.read_ubyte1()?,
                    self.reader.read_ubyte2()?,
                ),
                ConstantTag::MethodType => cp.set_method_type(index, self.reader.read_ubyte2()?),
                ConstantTag::InvokeDynamic => cp.set_invoke_dynamic(
                    index,
                    self.reader.read_ubyte2()?,
                    self.reader.read_ubyte2()?,
                ),
                _ => {
                    cp.set_invalid(index);
                    self.reader.skip(1);
                }
            };
            index += 1
        }
        Ok(cp)
    }

    fn parse_interfaces(
        &mut self,
        cp: &Handle<ConstantPool>,
        java_lang_class_bootstrapping: bool,
    ) -> Result<Handle<JArray>, ClassLoadErr> {
        let length = JInt::from(self.reader.read_ubyte2()?);
        if length == 0 {
            return Ok(Handle::new(self.vm.shared_objs().empty_sys_arr));
        }
        let interfaces = Handle::new(JArray::new_internal_permanent(length, Thread::current()));
        for index in 0..length {
            let class_name = cp.get_class_name(self.reader.read_ubyte2()?);
            let class = if java_lang_class_bootstrapping {
                class_name.cast()
            } else {
                let class = self
                    .vm
                    .bootstrap_class_loader
                    .load_class(class_name.as_str())?;
                if !class.class_data().is_interface() {
                    return Err(ClassLoadErr::VerifyFailed(
                        "class file format error: invalid interface".to_string(),
                    ));
                }
                class
            };
            interfaces.set(index, class.cast());
        }
        Ok(interfaces)
    }

    fn parse_fields(&mut self, cp: &Handle<ConstantPool>) -> Result<ParsedFields, ClassLoadErr> {
        let fields_count = self.reader.read_ubyte2()? as JInt;
        let mut static_fields_layout = FieldLayout::default();
        let mut inst_fields_layout = FieldLayout::default(); // TODO
        let thread = Thread::current();
        let fields = Handle::new(JArray::new_internal_permanent(fields_count, thread));
        for field_index in 0..fields_count {
            let access_flags = self.reader.read_ubyte2()?;
            let name_index = self.reader.read_ubyte2()?;
            let name = cp.get_utf8(name_index);
            debug_assert!(name.as_str().len() > 0);
            let descriptor_index = self.reader.read_ubyte2()?;
            let descriptor = cp.get_utf8(descriptor_index);
            let attrs_count = self.reader.read_ubyte2()?;
            let field_class_or_null: JClassPtr;
            let field_val_size: u16;
            let descriptor_symbol: SymbolPtr;

            match DescriptorParser::from_symbol(descriptor, self.vm).next() {
                Descriptor::ResolvedClass(decl_class, val_size) => {
                    field_class_or_null = decl_class;
                    field_val_size = val_size as _;
                    descriptor_symbol = descriptor;
                    log::trace!(
                        "ClassParser parsed: {}, decl_class, field {}, inst size {}, descriptor {}",
                        self.this_class_name.as_str(),
                        name.as_str(),
                        field_val_size,
                        descriptor.as_str()
                    );
                }
                Descriptor::Symbol(descriptor, val_size) => {
                    field_class_or_null = JClassPtr::null();
                    field_val_size = val_size as _;
                    descriptor_symbol = descriptor;
                    log::trace!(
                        "ClassParser parsed: {}, symbol, field {}, inst size {}, descriptor {}",
                        self.this_class_name.as_str(),
                        name.as_str(),
                        field_val_size,
                        descriptor.as_str()
                    );
                }
                _ => {
                    return Err(ClassLoadErr::InvalidFormat(format!(
                        "invalid descriptor for field {}",
                        name.as_str()
                    )))
                }
            };

            let field_offset;
            if FieldAccessFlags::is_static(access_flags) {
                field_offset = static_fields_layout.obtain_field_offset(field_val_size);
            } else {
                field_offset = inst_fields_layout.obtain_field_offset(field_val_size);
            }
            let mut field = Field::new(
                access_flags,
                field_offset,
                name,
                descriptor_symbol,
                field_class_or_null,
                thread,
            );

            for _attr_index in 0..attrs_count {
                let attr_name_index = self.reader.read_ubyte2()?;
                let attr_name = cp.get_utf8(attr_name_index);
                let attr_length = self.reader.read_ubyte4()?;
                let attr_name_str = (*attr_name).as_str();
                match attr_name_str {
                    "ConstantValue" => {
                        let constval_index = self.reader.read_ubyte2()?;
                        field.set_constval_index(constval_index);
                    }
                    // "Synthetic" => assert!(attr_length == 0),
                    "Signature" => {
                        self.reader.read_ubyte2()?; // signature_index(ignore)
                    }
                    // "Deprecated" => assert!(attr_length == 0),
                    "RuntimeVisibleAnnotations" => {
                        let num_annos = self.reader.read_ubyte2()?;
                        for _anno_index in 0..num_annos {
                            self.parse_annotation(cp)?;
                        }
                    }
                    "RuntimeInvisibleAnnotations" => {
                        let num_annos = self.reader.read_ubyte2()?;
                        for _anno_index in 0..num_annos {
                            self.parse_annotation(cp)?;
                        }
                    }
                    _ => {
                        self.reader.skip(attr_length as usize);
                    }
                }
            }
            fields.set(field_index, field.cast());
            // fields.set_field(field_index as isize, field);
        }
        let inst_size = inst_fields_layout.get_aligned_size();
        let static_fields_size = static_fields_layout.get_aligned_size();
        Ok(ParsedFields {
            fields,
            static_fields_size,
            inst_size: self
                .vm
                .shared_objs()
                .resize_for_metadata(self.this_class_name, inst_size),
            metadata_offset: inst_size,
        })
    }

    fn parse_annotation(&mut self, cp: &Handle<ConstantPool>) -> Result<(), ClassLoadErr> {
        let _type_index = self.reader.read_ubyte2()?;
        let num_element_value_pairs = self.reader.read_ubyte2()?;
        for _element_index in 0..num_element_value_pairs {
            self.parse_annotation_element(cp)?;
        }
        return Ok(());
    }

    fn parse_annotation_element(&mut self, cp: &Handle<ConstantPool>) -> Result<(), ClassLoadErr> {
        let _element_name_index = self.reader.read_ubyte2()?;
        let element_tag = self.reader.read_ubyte1()?;
        match element_tag as char {
            'B' | 'C' | 'D' | 'F' | 'I' | 'J' | 'S' | 'Z' | 's' => {
                let _constval_index = self.reader.read_ubyte2()?;
            }
            'e' => {
                let _type_name_index = self.reader.read_ubyte2()?;
                let _const_name_index = self.reader.read_ubyte2()?;
            }
            'c' => {
                let class_index = self.reader.read_ubyte2()?;
                let _class_name = cp.get_utf8(class_index);
            }
            '@' => {
                self.parse_annotation(cp)?;
            }
            '[' => {
                let num_values = self.reader.read_ubyte2()?;
                for _element_index in 0..num_values {
                    self.parse_annotation_element(cp)?;
                }
            }
            _ => {
                // TODO: unknown tag
            }
        }
        return Ok(());
    }

    fn parse_methods(
        &mut self,
        cp: &Handle<ConstantPool>,
        init_method: &mut MethodPtr,
    ) -> Result<Handle<JArray>, ClassLoadErr> {
        let methods_count = self.reader.read_ubyte2()? as JInt;
        let thread = Thread::current();
        let methods = Handle::new(JArray::new_internal_permanent(methods_count, thread));
        for index in 0..methods_count {
            let access_flags = self.reader.read_ubyte2()?;
            let name_index = self.reader.read_ubyte2()?;
            let name = cp.get_utf8(name_index);
            debug_assert!(name.as_str().len() > 0);
            let descriptor_index = self.reader.read_ubyte2()?;
            let descriptor = cp.get_utf8(descriptor_index);

            let mut descriptor_it = DescriptorParser::from_symbol(descriptor, self.vm);
            if Descriptor::OpenParenthesis != descriptor_it.next() {
                return Err(ClassLoadErr::InvalidFormat(format!(
                    "invalid method descriptor: {}, expected '('",
                    descriptor.as_str()
                )));
            }
            let mut params = Vec::new();
            let has_close_parenthesis: bool;
            'parse_descriptor: loop {
                let param_cls = match descriptor_it.next() {
                    Descriptor::ResolvedClass(resolved_cls, _) => resolved_cls,
                    Descriptor::Symbol(symbol, _) => {
                        if let Some(loaded_cls) = self
                            .vm
                            .bootstrap_class_loader
                            .find_class_with_symbol(symbol)
                        {
                            loaded_cls
                        } else {
                            symbol.cast()
                        }
                    }
                    Descriptor::CloseParenthesis => {
                        has_close_parenthesis = true;
                        break 'parse_descriptor;
                    }
                    Descriptor::OpenParenthesis
                    | Descriptor::InvalidDescriptor
                    | Descriptor::End => {
                        return Err(ClassLoadErr::InvalidFormat(format!(
                            "{}#{} invalid method descriptor: {}",
                            self.this_class_name.as_str(),
                            name.as_str(),
                            descriptor.as_str()
                        )));
                    }
                };
                debug_assert!(param_cls.is_not_null());
                params.push(param_cls);
            }
            if !has_close_parenthesis {
                return Err(ClassLoadErr::InvalidFormat(format!(
                    "invalid method descriptor: {}, expected ')'",
                    descriptor.as_str()
                )));
            }
            let params = if params.is_empty() {
                self.vm.shared_objs().empty_sys_arr
            } else {
                let method_params =
                    JArray::new_internal_permanent(params.len() as JInt, Thread::current());
                for i in 0..params.len() {
                    unsafe {
                        method_params.set(i as JInt, params.get_unchecked(i).cast());
                    }
                }
                method_params
            };

            let (ret_type, ret_descriptor) = match descriptor_it.next() {
                Descriptor::ResolvedClass(ret_type, _) => (ret_type, descriptor),
                Descriptor::Symbol(ret_descriptor, _) => (JClassPtr::null(), ret_descriptor),
                _ => {
                    return Err(ClassLoadErr::InvalidFormat(format!(
                        "invalid method descriptor: {}, expected return type",
                        descriptor.as_str()
                    )))
                }
            };
            let mut max_stack: u16 = 0;
            let mut max_locals: u16 = 0;
            let mut code_length: u16 = 0;
            let mut code: *const u8 = std::ptr::null();
            let mut ex_tab = Vec::new();

            let attrs_count = self.reader.read_ubyte2()?;
            for _attr_index in 0..attrs_count {
                let attr_name_index = self.reader.read_ubyte2()?;
                if attr_name_index > cp.length() {
                    return Err(ClassLoadErr::InvalidFormat(
                        "invalid method attr index".to_string(),
                    ));
                }
                let attr_length = self.reader.read_ubyte4()?;
                let attr_name = cp.get_utf8(attr_name_index);
                match attr_name.as_str() {
                    "Code" => {
                        max_stack = self.reader.read_ubyte2()?.try_into().unwrap();
                        max_locals = self.reader.read_ubyte2()?.try_into().unwrap();
                        self.parse_code(name, &mut code_length, &mut code)?;
                        ex_tab = self.parse_ex_tab(cp, name, code_length)?;
                        let code_attrs_count = self.reader.read_ubyte2()?;
                        for _code_attr_index in 0..code_attrs_count {
                            let _code_attr_name_index = self.reader.read_ubyte2()?;
                            let code_attr_length = self.reader.read_ubyte4()?;
                            self.reader.skip(code_attr_length as usize); // ignore attrs of the code
                        }
                    }
                    _ => {
                        self.reader.skip(attr_length as usize); // ignore all other attrs
                    }
                }
            }

            let method = Method::new(
                access_flags,
                name,
                descriptor,
                params,
                ret_type,
                ret_descriptor,
                max_stack,
                max_locals,
                code_length,
                code,
                &ex_tab,
                thread,
            );
            if name.as_str() == "<clinit>" {
                *init_method = method;
            }
            methods.set(index, method.cast());
        }
        return Ok(methods);
    }

    fn parse_code(
        &mut self,
        method_name: SymbolPtr,
        code_length: &mut u16,
        code: &mut *const u8,
    ) -> Result<(), ClassLoadErr> {
        let code_len = self.reader.read_ubyte4()?;
        if code_len >= 65536 {
            return Err(ClassLoadErr::InvalidFormat(format!(
                "{}#{}: invalid code_length",
                self.this_class_name.as_str(),
                method_name.as_str(),
            )));
        }
        assert!(self.reader.readable_length() >= code_len as usize);
        let code_buf = Ptr::from_raw(self.reader.available_buffer());
        *code_length = code_len as u16;
        *code = code_buf.as_raw_ptr();

        self.reader.skip(code_len as usize);
        Ok(())
    }

    fn parse_ex_tab(
        &mut self,
        cp: &Handle<ConstantPool>,
        method_name: SymbolPtr,
        code_length: u16,
    ) -> Result<Vec<ExceptionTable>, ClassLoadErr> {
        let ex_tab_length = self.reader.read_ubyte2()?;
        let mut result: Vec<ExceptionTable> = Vec::with_capacity(ex_tab_length as usize);
        for _ in 0..ex_tab_length {
            let start_pc = self.reader.read_ubyte2()?;
            if start_pc >= code_length {
                return Err(ClassLoadErr::InvalidFormat(format!(
                    "{}#{}: invalid exception_table",
                    self.this_class_name.as_str(),
                    method_name.as_str()
                )));
            }
            let end_pc = self.reader.read_ubyte2()?;
            if end_pc > code_length || start_pc >= end_pc {
                return Err(ClassLoadErr::InvalidFormat(format!(
                    "{}#{}: invalid exception_table",
                    self.this_class_name.as_str(),
                    method_name.as_str()
                )));
            }
            let handler_pc = self.reader.read_ubyte2()?;
            if handler_pc > code_length {
                return Err(ClassLoadErr::InvalidFormat(format!(
                    "{}#{}: invalid exception_table",
                    self.this_class_name.as_str(),
                    method_name.as_str()
                )));
            }
            let catch_type = self.reader.read_ubyte2()?;
            if catch_type != 0 {
                cp.get_class_name(catch_type);
            }
            result.push(ExceptionTable::new(
                start_pc, end_pc, handler_pc, catch_type,
            ));
        }
        return Ok(result);
    }

    fn parse_class_attrs(
        &mut self,
        cp: &Handle<ConstantPool>,
        mut _class: JClassPtr,
    ) -> Result<(), ClassLoadErr> {
        let class_attrs_count = self.reader.read_ubyte2()?;
        let thread = Thread::current();
        for _ in 0..class_attrs_count {
            let attr_name_index = self.reader.read_ubyte2()?;
            let attr_length = self.reader.read_ubyte4()?;
            let attr_name = cp.get_utf8(attr_name_index);
            match attr_name.as_str() {
                "InnerClasses" => {
                    let num_inners = self.reader.read_ubyte2()?;
                    let inners = JArray::new_internal_permanent(num_inners as JInt, thread);
                    for inner_idx in 0..num_inners {
                        let inner_class_info_index = self.reader.read_ubyte2()?;
                        let _outer_class_info_index = self.reader.read_ubyte2()?;
                        let _inner_name_index = self.reader.read_ubyte2()?;
                        let _inner_class_access_flags = self.reader.read_ubyte2()?;
                        inners.set(
                            inner_idx as JInt,
                            cp.get_class_name(inner_class_info_index as u16).cast(),
                        );
                    }
                    // class.set_inners(inners); TODO
                    continue;
                }
                "EnclosingMethod" => {
                    let class_index = self.reader.read_ubyte2()?;
                    let method_index = self.reader.read_ubyte2()?;
                    if method_index != 0 {
                        debug_assert_ne!(method_index, 0); // todo:
                    }
                    debug_assert_ne!(class_index, 0);
                    continue;
                }
                "Synthetic" => {
                    todo!()
                }
                "Signature" => {
                    let signature_index = self.reader.read_ubyte2()?;
                    let _signature = cp.get_utf8(signature_index);
                    // TODO: generic
                    continue;
                }
                "SourceFile" => {
                    self.reader.skip(attr_length as usize);
                    continue;
                }
                "SourceDebugExtension" => {
                    // TODO
                }
                "Deprecated" => {
                    // TODO
                }
                "RuntimeVisibleAnnotations" => {
                    // TODO
                }
                "RuntimeInvisibleAnnotations" => {
                    // TODO
                }
                "BootstrapMethods" => {}
                _ => {
                    return Err(ClassLoadErr::InvalidFormat(format!(
                        "unsupported attribute: {}",
                        attr_name.as_str()
                    )))
                }
            }

            self.reader.skip(attr_length as usize);
        }

        return Ok(());
    }

    fn major_version_is_support(major_version: u16) -> bool {
        match major_version {
            _m @ 45..=57 => true,
            _ => false,
        }
    }
}

struct ParsedFields {
    fields: Handle<JArray>,
    static_fields_size: u16,
    inst_size: u16,
    metadata_offset: u16,
}
