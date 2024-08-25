use core::str;
use std::mem::size_of;

use crate::{
    object::{
        prelude::{JBoolean, JByte, JChar, JDouble, JFloat, JInt, JLong, JShort},
        symbol::SymbolPtr,
    },
    vm::VM,
    JClassPtr, ObjectPtr,
};

pub struct DescriptorParser<'a> {
    value: &'a [u8],
    offset: usize,
    vm: &'a VM,
    cur_arr: bool,
}

impl<'a> DescriptorParser<'a> {
    pub fn from_symbol(symbol: SymbolPtr, vm: &'a VM) -> DescriptorParser<'a> {
        return Self::from_bytes(symbol.as_bytes(), vm);
    }

    pub fn from_bytes(value: &'a [u8], vm: &'a VM) -> DescriptorParser<'a> {
        return Self {
            value,
            offset: 0,
            vm,
            cur_arr: false,
        };
    }

    pub fn next(&mut self) -> Descriptor {
        let result = self.next_class(-1);
        self.cur_arr = false;
        return result;
    }

    fn read(&mut self) -> u8 {
        let prefix;
        unsafe {
            prefix = *self.value.get_unchecked(self.offset);
        }
        self.offset += 1;
        return prefix;
    }

    fn peek(&self) -> u8 {
        unsafe {
            return *self.value.get_unchecked(self.offset);
        }
    }

    fn next_class(&mut self, prev_offset: isize) -> Descriptor {
        if self.offset >= self.value.len() {
            return Descriptor::End;
        }
        let symbol_start = if prev_offset == -1 {
            self.offset
        } else {
            prev_offset as usize
        };
        let prefix = self.read();
        let preloaded_classes = self.vm.preloaded_classes();
        match prefix {
            b'B' => {
                return if prev_offset == -1 {
                    Descriptor::ResolvedClass(preloaded_classes.byte_cls(), size_of::<JByte>())
                } else {
                    Descriptor::Symbol(
                        self.class_symbol(symbol_start, self.offset - 1),
                        size_of::<ObjectPtr>(),
                    )
                }
            }
            b'C' => {
                return if prev_offset == -1 {
                    Descriptor::ResolvedClass(preloaded_classes.char_cls(), size_of::<JChar>())
                } else {
                    Descriptor::Symbol(
                        self.class_symbol(symbol_start, self.offset - 1),
                        size_of::<ObjectPtr>(),
                    )
                }
            }
            b'D' => {
                return if prev_offset == -1 {
                    Descriptor::ResolvedClass(preloaded_classes.double_cls(), size_of::<JDouble>())
                } else {
                    Descriptor::Symbol(
                        self.class_symbol(symbol_start, self.offset - 1),
                        size_of::<ObjectPtr>(),
                    )
                }
            }
            b'F' => {
                return if prev_offset == -1 {
                    Descriptor::ResolvedClass(preloaded_classes.float_cls(), size_of::<JFloat>())
                } else {
                    Descriptor::Symbol(
                        self.class_symbol(symbol_start, self.offset - 1),
                        size_of::<ObjectPtr>(),
                    )
                }
            }
            b'I' => {
                return if prev_offset == -1 {
                    Descriptor::ResolvedClass(preloaded_classes.int_cls(), size_of::<JInt>())
                } else {
                    Descriptor::Symbol(
                        self.class_symbol(symbol_start, self.offset - 1),
                        size_of::<ObjectPtr>(),
                    )
                }
            }
            b'J' => {
                return if prev_offset == -1 {
                    Descriptor::ResolvedClass(preloaded_classes.long_cls(), size_of::<JLong>())
                } else {
                    Descriptor::Symbol(
                        self.class_symbol(symbol_start, self.offset - 1),
                        size_of::<ObjectPtr>(),
                    )
                }
            }
            b'S' => {
                return if prev_offset == -1 {
                    Descriptor::ResolvedClass(preloaded_classes.short_cls(), size_of::<JShort>())
                } else {
                    Descriptor::Symbol(
                        self.class_symbol(symbol_start, self.offset - 1),
                        size_of::<ObjectPtr>(),
                    )
                }
            }
            b'Z' => {
                return if prev_offset == -1 {
                    Descriptor::ResolvedClass(preloaded_classes.bool_cls(), size_of::<JBoolean>())
                } else {
                    Descriptor::Symbol(
                        self.class_symbol(symbol_start, self.offset - 1),
                        size_of::<ObjectPtr>(),
                    )
                }
            }
            b'V' => {
                return if prev_offset == -1 {
                    Descriptor::ResolvedClass(preloaded_classes.void_cls(), 0)
                } else {
                    Descriptor::InvalidDescriptor
                }
            }
            b'L' => loop {
                if self.offset == self.value.len() {
                    return Descriptor::InvalidDescriptor;
                }
                if self.peek() != b';' {
                    self.offset += 1;
                } else {
                    // [Ljava/lang/Object; -> [Ljava/lang/Object;
                    // Ljava/lang/Object; -> java/lang/Object
                    let class_symbol;
                    if self.cur_arr {
                        class_symbol = self.class_symbol(symbol_start, self.offset);
                    } else {
                        class_symbol = self.class_symbol(symbol_start + 1, self.offset - 1);
                    }
                    self.offset += 1;
                    return Descriptor::Symbol(class_symbol, size_of::<ObjectPtr>());
                }
            },
            b'[' => {
                if prev_offset == -1 && self.value.len() == 2 {
                    match self.peek() {
                        b'B' => {
                            return Descriptor::ResolvedClass(
                                preloaded_classes.byte_arr_cls(),
                                size_of::<ObjectPtr>(),
                            )
                        }
                        b'C' => {
                            return Descriptor::ResolvedClass(
                                preloaded_classes.char_arr_cls(),
                                size_of::<ObjectPtr>(),
                            )
                        }
                        b'D' => {
                            return Descriptor::ResolvedClass(
                                preloaded_classes.double_arr_cls(),
                                size_of::<ObjectPtr>(),
                            )
                        }
                        b'F' => {
                            return Descriptor::ResolvedClass(
                                preloaded_classes.float_arr_cls(),
                                size_of::<ObjectPtr>(),
                            )
                        }
                        b'I' => {
                            return Descriptor::ResolvedClass(
                                preloaded_classes.int_arr_cls(),
                                size_of::<ObjectPtr>(),
                            )
                        }
                        b'J' => {
                            return Descriptor::ResolvedClass(
                                preloaded_classes.long_arr_cls(),
                                size_of::<ObjectPtr>(),
                            )
                        }
                        b'S' => {
                            return Descriptor::ResolvedClass(
                                preloaded_classes.short_arr_cls(),
                                size_of::<ObjectPtr>(),
                            )
                        }
                        b'Z' => {
                            return Descriptor::ResolvedClass(
                                preloaded_classes.bool_arr_cls(),
                                size_of::<ObjectPtr>(),
                            )
                        }
                        b'V' => return Descriptor::InvalidDescriptor,
                        _ => {}
                    }
                }
                self.cur_arr = true;
                return self.next_class(symbol_start as isize);
            }
            b')' => {
                return Descriptor::CloseParenthesis;
            }
            b'(' => {
                return Descriptor::OpenParenthesis;
            }
            _ => return Descriptor::InvalidDescriptor,
        }
    }

    fn class_symbol(&mut self, start: usize, end: usize) -> SymbolPtr {
        let class_name;
        unsafe {
            class_name = str::from_utf8_unchecked(&self.value[start..=end]);
        }
        return self.vm.symbol_table.get_or_insert(class_name);
    }
}

#[derive(PartialEq, Debug)]
pub enum Descriptor {
    ResolvedClass(JClassPtr, usize),
    Symbol(SymbolPtr, usize),
    OpenParenthesis,
    CloseParenthesis,
    InvalidDescriptor,
    End,
}

#[cfg(test)]
mod tests {
    use crate::{classfile::descriptor::Descriptor, memory::POINTER_SIZE, test::run_in_vm};

    use super::DescriptorParser;

    #[test]
    fn parse_primitive_descriptor() {
        let mut cfg = crate::vm::VMConfig::default();
        cfg.set_class_path("./tests/classes");
        run_in_vm("./tests/classes", |vm| {
            let mut descriptor = DescriptorParser::from_symbol(vm.get_symbol("IIJ"), &vm);
            assert_eq!(
                Descriptor::ResolvedClass(vm.preloaded_classes().int_cls(), 4),
                descriptor.next()
            );
            assert_eq!(
                Descriptor::ResolvedClass(vm.preloaded_classes().int_cls(), 4),
                descriptor.next()
            );
            assert_eq!(
                Descriptor::ResolvedClass(vm.preloaded_classes().long_cls(), 8),
                descriptor.next()
            );
            assert_eq!(Descriptor::End, descriptor.next());
        });
    }

    #[test]
    fn parse_ref_size_descriptor() {
        run_in_vm("./tests/classes", |vm| {
            let mut descriptor = DescriptorParser::from_symbol(
                vm.get_symbol(
                    "ILjava/lang/String;Ljava/lang/Object;[Ljava/lang/Object;[[[Ljava/lang/Object;DJ",
                ),
                &vm,
            );
            assert_eq!(
                Descriptor::ResolvedClass(vm.preloaded_classes().int_cls(), 4),
                descriptor.next()
            );
            assert_eq!(
                Descriptor::Symbol(
                    vm.symbol_table.get_or_insert("java/lang/String"),
                    POINTER_SIZE
                ),
                descriptor.next()
            );
            assert_eq!(
                Descriptor::Symbol(
                    vm.symbol_table.get_or_insert("java/lang/Object"),
                    POINTER_SIZE
                ),
                descriptor.next()
            );
            assert_eq!(
                Descriptor::Symbol(
                    vm.symbol_table.get_or_insert("[Ljava/lang/Object;"),
                    POINTER_SIZE
                ),
                descriptor.next()
            );
            assert_eq!(
                Descriptor::Symbol(
                    vm.symbol_table.get_or_insert("[[[Ljava/lang/Object;"),
                    POINTER_SIZE
                ),
                descriptor.next()
            );
            assert_eq!(
                Descriptor::ResolvedClass(vm.preloaded_classes().double_cls(), 8),
                descriptor.next()
            );
            assert_eq!(
                Descriptor::ResolvedClass(vm.preloaded_classes().long_cls(), 8),
                descriptor.next()
            );
            assert_eq!(Descriptor::End, descriptor.next());
        });
    }

    #[test]
    fn parse_method_descriptor() {
        run_in_vm("./tests/classes", |vm| {
            let mut descriptor = DescriptorParser::from_symbol(
                vm.get_symbol("(ILjava/lang/String;Ljava/lang/Object;[Ljava/lang/Object;[[[Ljava/lang/Object;IJ)V"),
                &vm,
            );
            assert_eq!(Descriptor::OpenParenthesis, descriptor.next());
            assert_eq!(
                Descriptor::ResolvedClass(vm.preloaded_classes().int_cls(), 4),
                descriptor.next()
            );
            assert_eq!(
                Descriptor::Symbol(
                    vm.symbol_table.get_or_insert("java/lang/String"),
                    POINTER_SIZE
                ),
                descriptor.next()
            );
            assert_eq!(
                Descriptor::Symbol(
                    vm.symbol_table.get_or_insert("java/lang/Object"),
                    POINTER_SIZE
                ),
                descriptor.next()
            );
            assert_eq!(
                Descriptor::Symbol(
                    vm.symbol_table.get_or_insert("[Ljava/lang/Object;"),
                    POINTER_SIZE
                ),
                descriptor.next()
            );
            assert_eq!(
                Descriptor::Symbol(
                    vm.symbol_table.get_or_insert("[[[Ljava/lang/Object;"),
                    POINTER_SIZE
                ),
                descriptor.next()
            );
            assert_eq!(
                Descriptor::ResolvedClass(vm.preloaded_classes().int_cls(), 4),
                descriptor.next()
            );
            assert_eq!(
                Descriptor::ResolvedClass(vm.preloaded_classes().long_cls(), 8),
                descriptor.next()
            );
            assert_eq!(Descriptor::CloseParenthesis, descriptor.next());
            assert_eq!(
                Descriptor::ResolvedClass(vm.preloaded_classes().void_cls(), 0),
                descriptor.next()
            );
            assert_eq!(Descriptor::End, descriptor.next());
        });
    }
}
