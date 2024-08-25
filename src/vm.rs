use crate::classfile::class_loader::BootstrapClassLoader;
use crate::classfile::ClassLoadErr;
use crate::memory::heap::Heap;
use crate::memory::Address;
use crate::native::builtin_natives::BuiltinNativeFunctions;
use crate::native::jni::JNIWrapper;
use crate::object::class::InitializationError;
use crate::object::method::MethodPtr;
use crate::object::prelude::Ptr;
use crate::object::string::{JStringPtr, Utf16String};
use crate::object::symbol::{StringTable, SymbolPtr, SymbolTable};
use crate::runtime::interpreter::Interpreter;
use crate::shared::{PreloadedClasses, SharedObjects};
use crate::thread::{Thread, ThreadManager, ThreadPtr};
use crate::value::JValue;
use crate::{utils, JClassPtr, ObjectPtr};
use std::path::{Path, PathBuf};

pub type VMPtr = Ptr<VM>;

#[derive(Clone)]
pub struct VMConfig {
    current_dir: String,
    rsvm_home: String,
    class_path: String,
    pub boot_lib_path: Option<String>,
    pub stack_size: usize,
    pub main_class: String,
}

impl VMConfig {
    pub fn current_dir(&self) -> &str {
        &self.current_dir
    }

    pub fn rsvm_home(&self) -> &str {
        &self.rsvm_home
    }

    pub fn set_rsvm_home(&mut self, rsvm_home: &str) {
        self.rsvm_home = rsvm_home.into();
    }

    pub fn class_path(&self) -> &str {
        &self.class_path
    }

    pub fn set_class_path(&mut self, cp: &str) {
        self.class_path = Self::build_class_path(&self.rsvm_home, cp);
    }

    pub fn boot_lib_path(&self) -> Option<&str> {
        self.boot_lib_path.as_ref().map(|s| s.as_str())
    }

    fn get_rsvm_home_from_os_env() -> Option<String> {
        if let Some(rsvm_home) = std::env::var_os("rsvm.home") {
            if let Ok(rsvm_home) = rsvm_home.into_string() {
                Some(rsvm_home)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get_rsvm_home(current_dir: &str) -> String {
        return if let Some(rsvm_home) = Self::get_rsvm_home_from_os_env() {
            rsvm_home
        } else if let Some(exe_dir_path) = Self::get_exe_dir_path() {
            return exe_dir_path.display().to_string();
        } else {
            current_dir.into()
        };
    }

    fn get_exe_dir_path() -> Option<PathBuf> {
        match std::env::current_exe() {
            Ok(mut exe_path) => {
                exe_path.pop();
                return Some(exe_path);
            }
            Err(_) => None,
        }
    }

    fn get_rt_jar_path(rsvm_home: &str) -> String {
        let mut rt_jar_path = PathBuf::from(Path::new(rsvm_home));
        rt_jar_path.push("lib");
        rt_jar_path.push("rt.jar");
        return rt_jar_path.display().to_string();
    }

    fn get_charsets_jar_path(rsvm_home: &str) -> String {
        let mut charsets_jar = PathBuf::from(Path::new(rsvm_home));
        charsets_jar.push("lib");
        charsets_jar.push("charsets.jar");
        return charsets_jar.display().to_string();
    }

    fn build_class_path(rsvm_home: &str, cp: &str) -> String {
        let mut class_path = String::new();
        let rt_jar_path = Self::get_rt_jar_path(rsvm_home);
        let charsets_jar_path = Self::get_charsets_jar_path(rsvm_home);
        class_path.push_str(&rt_jar_path);
        class_path.push_str(utils::get_path_separator());
        class_path.push_str(&charsets_jar_path);
        class_path.push_str(utils::get_path_separator());
        class_path.push_str(cp);
        return class_path;
    }
}

impl Default for VMConfig {
    fn default() -> Self {
        let current_dir = std::env::current_dir().unwrap();
        let current_dir = if let Some(dir) = current_dir.to_str() {
            dir
        } else {
            ""
        }
        .to_string();
        let rsvm_home = Self::get_rsvm_home(&current_dir);
        let class_path = Self::build_class_path(&rsvm_home, ".");
        Self {
            current_dir,
            rsvm_home,
            class_path,
            boot_lib_path: None,
            stack_size: 2 * crate::memory::MB,
            main_class: "Main".to_string(),
        }
    }
}

#[derive(Debug)]
pub enum VMError {
    InitError(String),
    ClassLoaderErr(ClassLoadErr),
    ClassInitError(InitializationError),
    RuntimeError(String),
}

pub struct VM {
    pub bootstrap_class_loader: BootstrapClassLoader,
    heap: Heap,
    preloaded_classes: PreloadedClasses,
    shared_objs: SharedObjects,
    builtin_native_fns: BuiltinNativeFunctions,
    jni: JNIWrapper,
    pub(crate) symbol_table: SymbolTable,
    pub(crate) string_table: StringTable,
    pub(crate) thread_mgr: ThreadManager,
    pub(crate) cfg: VMConfig,
}

impl VM {
    pub fn new(cfg: &VMConfig) -> VMPtr {
        crate::os::init();
        let vm = Box::new(VM {
            bootstrap_class_loader: BootstrapClassLoader::default(),
            heap: Heap::new(),
            preloaded_classes: PreloadedClasses::new(),
            shared_objs: SharedObjects::default(),
            builtin_native_fns: BuiltinNativeFunctions::new(),
            jni: JNIWrapper::default(),
            symbol_table: SymbolTable::default(),
            string_table: StringTable::default(),
            thread_mgr: ThreadManager::new(),
            cfg: cfg.clone(),
        });
        return VMPtr::from_raw(Box::into_raw(vm));
    }

    pub fn init(&mut self) -> Result<(), VMError> {
        self.init_vm()?;
        return Ok(());
    }

    pub fn destroy(&self) {
        self.heap.destroy();
    }

    pub fn as_ptr(&self) -> VMPtr {
        VMPtr::from_ref(self)
    }

    pub fn get_symbol(&self, symbol: &str) -> SymbolPtr {
        return self.symbol_table.get_or_insert(symbol);
    }

    pub fn get_intern_jstr(&self, val: &Utf16String, thread: ThreadPtr) -> JStringPtr {
        return self.string_table.get_or_insert_str(val, thread);
    }

    pub fn get_symbol_with_jstr(&self, jstr: JStringPtr) -> SymbolPtr {
        return self.symbol_table.get_with_jstr(jstr);
    }

    pub fn get_jstr_from_symbol(&self, symbol: SymbolPtr, thread: ThreadPtr) -> JStringPtr {
        return self.string_table.from_symbol(symbol, thread);
    }

    pub fn intern_jstr(&self, jstr: JStringPtr, thread: ThreadPtr) -> JStringPtr {
        return self.string_table.intern_jstr(jstr, thread);
    }

    pub fn get_static_method(
        &self,
        class: JClassPtr,
        method_name: &str,
        descriptor: &str,
        thread: ThreadPtr,
    ) -> Result<MethodPtr, VMError> {
        class
            .initialize(thread)
            .map_err(|e| VMError::ClassInitError(e))?;
        let method = self.get_symbol(method_name);
        let descriptor = self.get_symbol(descriptor);
        if let Ok(resolved_method) = class.resolve_self_method(method, descriptor) {
            return Ok(resolved_method.method);
        }
        return Err(VMError::RuntimeError(
            format!("method {} not found", method_name).into(),
        ));
    }

    pub fn get_method(
        &self,
        class: JClassPtr,
        method_name: &str,
        descriptor: &str,
        thread: ThreadPtr,
    ) -> Result<MethodPtr, VMError> {
        class
            .initialize(thread)
            .map_err(|e| VMError::ClassInitError(e))?;
        let method = self.get_symbol(method_name);
        let descriptor = self.get_symbol(descriptor);
        if let Ok(resolved_method) = class.resolve_class_method(method, descriptor, self) {
            return Ok(resolved_method.method);
        }
        return Err(VMError::RuntimeError(
            format!("method {} not found", method_name).into(),
        ));
    }

    pub fn call_static_void(&self, class: JClassPtr, method: MethodPtr, args: &[JValue]) {
        let thread = Thread::current();
        class.initialize(thread).unwrap();
        Interpreter::call_static_method(class, method, args, thread);
    }

    pub fn call_static(&self, class: JClassPtr, method: MethodPtr, args: &[JValue]) -> JValue {
        let thread = Thread::current();
        class.initialize(thread).unwrap();
        return Interpreter::call_static_method(class, method, args, thread);
    }

    pub fn call_obj_void(&self, objref: ObjectPtr, method: MethodPtr, args: &[JValue]) {
        let thread = Thread::current();
        Interpreter::call_obj_void_method(objref, method, args, thread);
    }

    pub fn call_obj(&self, objref: ObjectPtr, method: MethodPtr, args: &[JValue]) -> JValue {
        let thread = Thread::current();
        return Interpreter::call_obj_method(objref, method, args, thread);
    }

    pub(crate) fn heap(&self) -> &Heap {
        return &self.heap;
    }

    fn init_vm(&mut self) -> Result<(), VMError> {
        // let vm = Self::new(cfg)?;
        Thread::attach_current_thread(self);

        self.heap().debug("==========");

        let thread = Thread::current();

        self.symbol_table = SymbolTable::new(thread);
        self.string_table = StringTable::new(thread);

        self.bootstrap_class_loader =
            BootstrapClassLoader::new(&self.cfg.class_path, &self.cfg.current_dir, thread);

        let vm = VMPtr::from_ref(self);
        self.jni.init(vm);
        self.shared_objs.init(thread);
        self.preloaded_classes.init(vm, thread)?;
        self.shared_objs.post_init(vm, thread)?;

        // global::classes::init(self).map_err(|e| VMError::ClassLoaderErr(e))?;

        return Ok(());
    }

    #[inline]
    pub(crate) fn preloaded_classes(&self) -> &PreloadedClasses {
        &self.preloaded_classes
    }

    #[inline]
    pub(crate) fn shared_objs(&self) -> &SharedObjects {
        &self.shared_objs
    }

    pub(crate) fn get_builtin_native_fn(&self, fn_name: &str) -> Option<Address> {
        return self.builtin_native_fns.get_builtin_native_fn(fn_name);
    }

    pub(crate) fn jni(&self) -> &JNIWrapper {
        &self.jni
    }
}

unsafe impl Send for VM {}
unsafe impl Send for VMPtr {}

#[cfg(test)]
mod tests {
    use crate::{object::string::JString, test, thread::Thread, value::JValue, JArray};

    #[test]
    fn invoke_hello_rsvm() {
        test::run_in_vm_and_call_static(
            "./tests/classes",
            "rsvm.HelloRSVM",
            "main",
            "([Ljava/lang/String;)V",
            |_| {
                vec![JValue::with_arr_val(JArray::new_obj_arr(
                    1,
                    Thread::current(),
                ))]
            },
            |_, _| {},
        );
    }

    #[test]
    fn invoke_fibonacci() {
        test::run_in_vm_and_call_static(
            "./tests/classes",
            "rsvm.MethodCall",
            "fibonacci",
            "(I)I",
            |_| vec![JValue::with_int_val(32)],
            |_, result| {
                assert_eq!(rs_fibonacci(32), result.int_val());
            },
        );
    }

    #[test]
    fn invoke_virtual() {
        test::run_in_vm_and_call_static(
            "./tests/classes",
            "rsvm.MethodCall",
            "invokeVirtual",
            "()Ljava/lang/String;",
            |_| vec![],
            |vm, result| {
                let result = result.obj_val().cast::<JString>();
                let result = JString::to_rust_string(result, vm.as_ref());
                assert_eq!("Sub", &result);
            },
        );
    }

    const fn rs_fibonacci(num: i32) -> i32 {
        if num == 1 || num == 2 {
            return 1;
        }
        return rs_fibonacci(num - 1) + rs_fibonacci(num - 2);
    }
}
