use std::{path::PathBuf, process::Command, str::FromStr};

use crate::{
    thread::Thread,
    value::JValue,
    vm::{VMConfig, VMPtr, VM},
};

pub fn run_in_vm_and_call_static<
    ArgFn: 'static + Send + FnOnce(VMPtr) -> Vec<JValue>,
    Action: 'static + Send + FnOnce(VMPtr, JValue),
>(
    class_path: &'static str,
    class_name: &'static str,
    method_name: &'static str,
    method_descriptor: &'static str,
    args_fn: ArgFn,
    f: Action,
) {
    ensure_class_exists(class_path, class_name);
    run_in_vm(class_path, move |vm| {
        let class = vm
            .bootstrap_class_loader
            .load_binary_name_class(class_name)
            .unwrap();

        let method = vm
            .get_static_method(class, method_name, method_descriptor, Thread::current())
            .unwrap();
        let ret_val = vm.call_static(class, method, args_fn(vm).as_slice());
        f(vm, ret_val);
    });
}

pub fn run_in_vm<F: FnOnce(VMPtr) + Send + 'static>(class_path: &str, f: F) {
    let attached = Thread::current();
    if attached.is_not_null() {
        run_in_new_thread(attached.vm_ptr(), true, f);
        return;
    }

    let mut cfg = VMConfig::default();
    let mut rsvm_home = PathBuf::from_str(cfg.rsvm_home()).unwrap();
    rsvm_home.pop();
    cfg.set_rsvm_home(&rsvm_home.display().to_string());
    cfg.set_class_path(class_path);
    let vm = VM::new(&cfg);

    Thread::attach_current_thread(vm.as_ref());

    run_in_new_thread(vm, true, |vm| {
        Thread::attach_current_thread(vm.as_ref());
        vm.as_mut_ref().init().unwrap();

        f(vm);
    });
}

fn run_in_new_thread<F: FnOnce(VMPtr) + Send + 'static>(vm: VMPtr, join: bool, f: F) {
    let thread = std::thread::Builder::new()
        .name("test".into())
        .stack_size(4 * 1024 * 1024)
        .spawn(move || {
            f(vm);
        })
        .unwrap();
    if join {
        thread.join().unwrap();
    }
}

fn get_real_file_path(class_path: &str, file: &str) -> PathBuf {
    let mut result = PathBuf::from_str(class_path).unwrap();
    result.push(&file);
    return result;
}

fn get_file_with_suffix(class_name: &str, suffix: &str) -> String {
    let mut file = class_name.replace(".", "/");
    file.push_str(suffix);
    return file;
}

fn ensure_class_exists(class_path: &str, class_name: &str) {
    let java_file = get_file_with_suffix(class_name, ".java");
    let java_file_path = get_real_file_path(class_path, &java_file);
    if !java_file_path.exists() {
        panic!("{} not exists", java_file_path.display().to_string());
    }
    let class_file = get_file_with_suffix(class_name, ".class");
    let class_file_path = get_real_file_path(class_path, &class_file);

    if !class_file_path.exists() {
        let mut work_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        work_dir.push(class_path);

        let mut cmd = Command::new("javac")
            .arg("-target")
            .arg("1.7")
            .arg("-source")
            .arg("1.7")
            .arg("-cp")
            .arg(".")
            .arg(&java_file)
            .current_dir(work_dir.display().to_string())
            .spawn()
            .expect("javac command failed to start");
        cmd.wait().unwrap();

        if !class_file_path.exists() {
            panic!("{:#?}, {} not found", work_dir, class_file);
        }
    }
}
