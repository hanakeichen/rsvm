use super::reader::ClassReader;
use super::{ClassLoadErr, ClassLoader};
use crate::handle::Handle;
use crate::object::class::{ConstantPool, ConstantTag, Field, FieldArray, Method, MethodArray};
use crate::object::prelude::*;
use crate::vm;
use std::f64;

const CLASS_FILE_MAGIC: u32 = 0xCAFEBABE;

pub struct ClassParser<'a> {
    class_loader: &'a mut dyn ClassLoader,
    reader: Box<dyn ClassReader>,
}

impl<'a> ClassParser<'a> {
    pub fn parse_class(&mut self) -> Result<Handle<Class>, ClassLoadErr> {
        let magic = self.reader.read_ubyte4()?;
        if magic != CLASS_FILE_MAGIC {
            return Err("cannot identify the magic number");
        }
        let minor_version = self.reader.read_ubyte2()?;
        let major_version = self.reader.read_ubyte2()?;
        if !Self::major_version_is_support(major_version) {
            return Err("unsupported class file version");
        }
        let cp = self.parse_constant_pool()?;
        let access_flags = self.reader.read_ubyte2()?;
        let this_class = self.reader.read_ubyte2()?;
        let class_name = cp.get_utf8(this_class);
        let super_class_index = self.reader.read_ubyte2()?;
        let super_class_name = cp.get_utf8(super_class_index);
        let super_class = self.class_loader.resolve_class(super_class_name)?;
        let interfaces = self.parse_interfaces(&cp)?;
        let fields = self.parse_fields(&cp)?;
        let methods = self.parse_methods(&cp)?;
        // let attrs = self.parse_attributes(cp)?;
        let class_attrs_count = self.reader.read_ubyte2()?;
        for class_attr_index in 0..class_attrs_count {
            let attr_name_index = self.reader.read_ubyte2()?;
            let attr_length = self.reader.read_ubyte2()?;
            self.reader.skip(attr_length as usize);
        }
        let class = Handle::new(Class::new_permanent(
            cp.as_ptr(),
            access_flags,
            class_name,
            super_class.as_ptr(),
            interfaces.as_ptr(),
            fields.as_ptr(),
            methods.as_ptr(),
        ));
        return Ok(class);

        /* Ok(Class {
            magic,
            minor_version,
            major_version,
            cp,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attrs,
        }) */
    }

    /// jvms-4.4
    fn parse_constant_pool(&mut self) -> Result<Handle<ConstantPool>, ClassLoadErr> {
        let cp_count = self.reader.read_ubyte2()?;
        // The constant_pool table is indexed from 1 to constant_pool_count - 1.
        let mut cp = Handle::new(ConstantPool::new(cp_count));
        // let cp = &*cp_handle;
        for index in 1..cp_count {
            let tag = ConstantTag::from(self.reader.read_ubyte1()?);
            match tag {
                ConstantTag::Utf8 => {
                    let length = self.reader.read_ubyte2()?;
                    let mut symbol = String::with_capacity(length as usize);
                    for _ in 0..length {
                        symbol.push(self.reader.read_ubyte1()? as char);
                    }
                    cp.set_utf8(index, vm::instance().symbol_table.get_or_insert(symbol));
                }
                ConstantTag::Integer => cp.set_int32(index, self.reader.read_ubyte4()? as i32),
                ConstantTag::Float => cp.set_float(index, self.reader.read_ubyte4()? as f32),
                ConstantTag::Long => {
                    let high_bytes = (self.reader.read_ubyte4()? as JLong) << 32;
                    let low_bytes = self.reader.read_ubyte4()? as JLong;
                    cp.set_long(index, high_bytes | low_bytes);
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
                ConstantTag::InterfaceMethodref => cp.set_method_ref(
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
                _ => cp.set_invalid(index),
            };
        }
        Ok(cp)
    }

    fn parse_interfaces(
        &mut self,
        cp: &Handle<ConstantPool>,
    ) -> Result<Handle<JRefArray>, ClassLoadErr> {
        let length = self.reader.read_ubyte2()?;
        let interfaces = Handle::new(JRefArray::new_obj_permanent(length as JInt));
        for index in 0..length {
            let class_name = cp.get_class_name(self.reader.read_ubyte2()?);
            let class = self.class_loader.resolve_class(class_name)?;
            if !class.is_interface() {
                return Err("class file format error");
            }
            interfaces.set(index as JInt, class.value().cast());
        }
        Ok(interfaces)
    }

    fn parse_fields(
        &mut self,
        cp: &Handle<ConstantPool>,
    ) -> Result<Handle<FieldArray>, ClassLoadErr> {
        let fields_count = self.reader.read_ubyte2()?;
        let mut fields = Handle::new(FieldArray::new(fields_count));
        for field_index in 0..fields_count {
            let access_flags = self.reader.read_ubyte2()?;
            let name_index = self.reader.read_ubyte2()?;
            let name = cp.get_utf8(name_index);
            let descriptor_index = self.reader.read_ubyte2()?;
            let descriptor = cp.get_utf8(descriptor_index);
            let attrs_count = self.reader.read_ubyte2()?;

            let mut field = Field::new(access_flags, name, descriptor);

            for attr_index in 0..attrs_count {
                let attr_name_index = self.reader.read_ubyte2()?;
                let attr_name = cp.get_utf8(attr_name_index);
                let attr_length = self.reader.read_ubyte2()?;
                let attr_name_str = unsafe { (*attr_name).as_str() };
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
                        for anno_index in 0..num_annos {
                            self.parse_annotation(cp)?;
                        }
                    }
                    "RuntimeInvisibleAnnotations" => {
                        let num_annos = self.reader.read_ubyte2()?;
                        for anno_index in 0..num_annos {
                            self.parse_annotation(cp)?;
                        }
                    }
                    _ => {
                        self.reader.skip(attr_length as usize);
                    }
                }
            }
            fields.set_field(field_index, field);
        }
        Ok(fields)
    }

    fn parse_annotation(&mut self, cp: &Handle<ConstantPool>) -> Result<(), ClassLoadErr> {
        let type_index = self.reader.read_ubyte2()?;
        let num_element_value_pairs = self.reader.read_ubyte2()?;
        for element_index in 0..num_element_value_pairs {
            self.parse_annotation_element(cp)?;
        }
        return Ok(());
    }

    fn parse_annotation_element(&mut self, cp: &Handle<ConstantPool>) -> Result<(), ClassLoadErr> {
        let element_name_index = self.reader.read_ubyte2()?;
        let element_tag = self.reader.read_ubyte1()?;
        match element_tag as char {
            'B' | 'C' | 'D' | 'F' | 'I' | 'J' | 'S' | 'Z' | 's' => {
                let constval_index = self.reader.read_ubyte2()?;
            }
            'e' => {
                let type_name_index = self.reader.read_ubyte2()?;
                let const_name_index = self.reader.read_ubyte2()?;
            }
            'c' => {
                let class_index = self.reader.read_ubyte2()?;
                let class_name = cp.get_utf8(class_index);
            }
            '@' => {
                self.parse_annotation(cp)?;
            }
            '[' => {
                let num_values = self.reader.read_ubyte2()?;
                for element_index in 0..num_values {
                    self.parse_annotation_element(cp)?;
                }
            }
            _ => {
                // TODO: report unknown tag error
            }
        }
        return Ok(());
    }

    fn parse_methods(
        &mut self,
        cp: &Handle<ConstantPool>,
    ) -> Result<Handle<MethodArray>, ClassLoadErr> {
        let methods_count = self.reader.read_ubyte2()?;
        let mut methods = Handle::new(MethodArray::new(methods_count));
        for index in 0..methods_count {
            let access_flags = self.reader.read_ubyte2()?;
            let name_index = self.reader.read_ubyte2()?;
            let name = cp.get_utf8(name_index);
            let descriptor_index = self.reader.read_ubyte2()?;
            let descriptor = cp.get_utf8(descriptor_index);

            let mut method = Method::new(access_flags, name, descriptor);

            let attrs_count = self.reader.read_ubyte2()?;
            for attr_index in 0..attrs_count {
                let attr_name_index = self.reader.read_ubyte2()?;
                if attr_name_index > cp.size() {
                    return Err("invalid constant pool index");
                }
                let attr_length = self.reader.read_ubyte4()?;
                let attr_name = cp.get_utf8(attr_name_index).as_str();
                match attr_name.as_ref() {
                    "Code" => {
                        method.set_max_stack(self.reader.read_ubyte2()?);
                        method.set_max_locals(self.reader.read_ubyte2()?);
                        let code = self.parse_code()?;
                        method.set_code(code.value());
                        self.parse_ex_tab()?; // TODO
                        let code_attrs_count = self.reader.read_ubyte2()?;
                        for code_attr_index in 0..code_attrs_count {
                            let code_attr_name_index = self.reader.read_ubyte2()?;
                            let code_attr_length = self.reader.read_ubyte4()?;
                            self.reader.skip(code_attr_length as usize); // ignore attrs of the code
                        }
                    }
                    _ => {
                        self.reader.skip(attr_length as usize); // ignore all other attrs
                    }
                }
            }
            methods.set_method(index, method);
        }
        return Ok(methods);
    }

    fn parse_code(&mut self) -> Result<Handle<JByteArray>, ClassLoadErr> {
        let code_length = self.reader.read_ubyte4()?;
        let code = Handle::new(JByteArray::new_permanent(code_length as i32));
        assert!(self.reader.readable_length() >= code_length as usize);
        unsafe {
            std::ptr::copy(
                self.reader.buffer(),
                code.raw_data().as_mut_ptr().cast(),
                code_length as usize,
            );
        }
        Ok(code)
    }

    fn parse_ex_tab(&mut self) -> Result<(), ClassLoadErr> {
        let ex_tab_length = self.reader.read_ubyte2()?;
        for _ in 0..ex_tab_length {
            let start_pc = self.reader.read_ubyte2()?;
            let end_pc = self.reader.read_ubyte2()?;
            let handler_pc = self.reader.read_ubyte2()?;
            let catch_type = self.reader.read_ubyte2()?;
        }
        Ok(())
    }

    fn major_version_is_support(major_version: u16) -> bool {
        match major_version {
            _m @ 45..=57 => true,
            _ => false,
        }
    }
}
