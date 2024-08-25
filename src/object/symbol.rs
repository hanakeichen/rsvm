use std::sync::{Mutex, RwLock};

use crate::{
    handle::{Handle, HandleScope},
    memory::Address,
    thread::{Thread, ThreadPtr},
    JArray, ObjectPtr,
};

use super::{
    array::JCharArrayPtr,
    hash_table::{GetEntryWithKey, HashTable, HashTablePtr, InsertNewWithKey},
    prelude::JInt,
    ptr::Ptr,
    string::{HeapString, JString, JStringPtr, Utf8String, Utf16String},
    Object,
};

pub type Symbol = HeapString;

pub type SymbolPtr = Ptr<Symbol>;

#[derive(Default)]
pub(crate) struct SymbolTable {
    table: RwLock<HashTablePtr>,
}

impl SymbolTable {
    pub fn new(thread: ThreadPtr) -> Self {
        Self {
            table: RwLock::new(HashTable::new(thread)),
        }
    }

    pub fn get_or_insert(&self, content: &str) -> SymbolPtr {
        let mut locked_table = self.table.write().expect("SymbolTable locked failed");
        let (table, symbol) =
            locked_table.get_or_insert_str(Utf8String::from(content), Thread::current());
        *locked_table = table;
        return symbol;
    }

    pub fn get_with_jstr(&self, jstr: JStringPtr) -> SymbolPtr {
        let locked_table = self.table.write().expect("SymbolTable locked failed");
        return locked_table.get_value_by_str_unchecked(jstr);
    }
}

impl<'a> GetEntryWithKey<Utf8String<'a>> for Symbol {
    fn hash_key(ref_str: Utf8String) -> JInt {
        return Symbol::hash_utf8(ref_str.value);
    }

    fn entry_equals_key(value: crate::memory::Address, ref_str: Utf8String) -> bool {
        let symbol: SymbolPtr = Ptr::from_addr(value);
        return symbol.equals_utf8(ref_str);
    }
}

impl<'a> GetEntryWithKey<JStringPtr> for Symbol {
    fn hash_key(ref_str: JStringPtr) -> JInt {
        return ref_str.cast::<Object>().hash();
    }

    fn entry_equals_key(value: crate::memory::Address, ref_str: JStringPtr) -> bool {
        let symbol: SymbolPtr = Ptr::from_addr(value);
        let chars = JString::get_char_array(ref_str, Thread::current().vm());
        return symbol.equals_utf16_ptr(chars.data(), chars.length());
    }
}

impl<'a> InsertNewWithKey<Utf8String<'a>, Symbol> for Symbol {
    fn new_entry_with_key(ref_str: Utf8String, key_hash: JInt, thread: ThreadPtr) -> Ptr<Symbol> {
        let symbol = Symbol::new_with_hash(ref_str.value, key_hash, thread);
        // log::trace!("new_with_ref_str symbol {}, {:x}", ref_str.value, symbol.as_usize());
        return symbol;
    }
}

#[derive(Default)]
pub(crate) struct StringTable {
    table: Mutex<HashTablePtr>,
}

impl StringTable {
    pub(crate) fn new(thread: ThreadPtr) -> Self {
        Self {
            table: Mutex::new(HashTable::new(thread)),
        }
    }

    pub(crate) fn get_or_insert_str(&self, val: &Utf16String, thread: ThreadPtr) -> JStringPtr {
        let mut locked_table = self.table.lock().expect("StringTable lock failed");
        let (new_table, intern_jstr) = locked_table.get_or_insert_str(val, thread);
        *locked_table = new_table;
        return intern_jstr;
    }

    pub(crate) fn intern_jstr(&self, jstr: JStringPtr, thread: ThreadPtr) -> JStringPtr {
        let chars = thread
            .vm()
            .shared_objs()
            .class_infos()
            .java_lang_string_info()
            .get_chars(jstr);
        let mut locked_table = self.table.lock().expect("StringTable lock failed");
        let (new_table, intern_jstr) = locked_table.get_or_insert_str(chars, thread);
        *locked_table = new_table;
        return intern_jstr;
    }

    pub(crate) fn from_symbol(&self, symbol: SymbolPtr, thread: ThreadPtr) -> JStringPtr {
        let mut locked_table = self.table.lock().expect("StringTable lock failed");
        if let Some(jstr) = locked_table.get_value_by_str(symbol) {
            return jstr;
        }
        let utf16_str = symbol.to_utf16();
        let utf16_len = utf16_str.len() as JInt;
        let _scope = HandleScope::new(thread);
        let mut chars_handle = Handle::new_with_thread(JCharArrayPtr::null(), thread);
        let value: JCharArrayPtr = JArray::new_permanent(
            utf16_len,
            thread.vm().preloaded_classes().char_arr_cls(),
            thread,
        )
        .cast();
        chars_handle.set_value(value);
        JString::char_arr_set_utf16_unchecked(value, &utf16_str, utf16_len);
        let result_obj: JStringPtr = thread
            .vm()
            .shared_objs()
            .class_infos()
            .java_lang_string_info()
            .create_permanent_with_chars(value.cast(), symbol.hash_code(), thread)
            .cast();
        *locked_table = locked_table.insert(result_obj, thread);
        return result_obj;
    }
}

impl GetEntryWithKey<&Utf16String> for JString {
    fn hash_key(content: &Utf16String) -> JInt {
        return HeapString::hash_utf16_str(content);
    }

    fn entry_equals_key(value: Address, content: &Utf16String) -> bool {
        let value = JStringPtr::from_addr(value);
        return JString::equals_utf16(value, content, Thread::current().vm());
    }
}

impl InsertNewWithKey<&Utf16String, JString> for JString {
    fn new_entry_with_key(ref_str: &Utf16String, key_hash: JInt, thread: ThreadPtr) -> Ptr<JString> {
        return thread
            .vm()
            .shared_objs()
            .class_infos()
            .java_lang_string_info()
            .create_permanent_with_utf16_hash(ref_str, key_hash, thread)
            .get_ptr();
    }
}

impl GetEntryWithKey<SymbolPtr> for JString {
    fn hash_key(ref_str: SymbolPtr) -> JInt {
        return ref_str.hash_code();
    }

    fn entry_equals_key(value: Address, ref_str: SymbolPtr) -> bool {
        // log::trace!(
        //     "GetFromRefString for JString, val addr: 0x{:x}",
        //     value.as_isize()
        // );
        debug_assert!(ObjectPtr::from_addr(value).jclass().name().as_str() == "java/lang/String");
        let chars = JString::get_char_array(JStringPtr::from_addr(value), Thread::current().vm());
        // log::trace!(
        //     "JString compare val {:#?}, symbol {:#?}",
        //     chars.data().as_slice(chars.length() as usize),
        //     ref_str.as_bytes()
        // );
        return ref_str.equals_utf16_ptr(chars.data(), chars.length());
    }
}

impl GetEntryWithKey<JCharArrayPtr> for JString {
    fn hash_key(ref_str: JCharArrayPtr) -> JInt {
        let chars: Ptr<u16> = ref_str.data().cast();
        return HeapString::hash_utf16_ptr(chars, ref_str.length());
    }

    fn entry_equals_key(value: Address, ref_str: JCharArrayPtr) -> bool {
        let thread = Thread::current();
        let vm = thread.vm();
        let chars = JString::get_char_array(JStringPtr::from_addr(value), vm);
        return JString::equals_chars(chars, ref_str);
    }
}

impl InsertNewWithKey<JCharArrayPtr, JString> for JString {
    fn new_entry_with_key(ref_str: JCharArrayPtr, key_hash: JInt, thread: ThreadPtr) -> Ptr<JString> {
        return thread
            .vm()
            .shared_objs()
            .class_infos()
            .java_lang_string_info()
            .create_permanent_with_chars(ref_str, key_hash, thread);
    }
}
