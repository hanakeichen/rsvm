use crate::handle::{Handle, HandleData, HandleScope};
use crate::memory::heap::{Heap, HeapPtr};
use crate::memory::lab::LocalAllocBuf;
use crate::object::prelude::{JInt, ObjectPtr, Ptr};
use crate::object::Object;
use crate::runtime::interpreter::Interpreter;
use crate::vm::{VMPtr, VM};
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::RwLock;

pub type ThreadPtr = Ptr<Thread>;

thread_local! {
    static VM_THREAD: Cell<ThreadPtr> = Cell::new(ThreadPtr::null());
}

pub struct ThreadManager {
    threads: RwLock<HashMap<u64, Box<Thread>>>,
}

impl ThreadManager {
    pub fn new() -> ThreadManager {
        let threads = RwLock::new(HashMap::new());

        return ThreadManager { threads };
    }

    pub fn add_thread(&self, thread: Box<Thread>) {
        let thread_id = thread.thread_id();
        let mut threads = self
            .threads
            .write()
            .expect("cannot add thread on the thread manager");
        threads.insert(thread_id, thread);
    }

    pub fn remove_thread(&self, thread_id: u64) {
        let mut threads = self
            .threads
            .write()
            .expect("cannot remove thread on the thread manager");
        threads.remove(&thread_id);
    }
}

pub struct Thread {
    _handle_scope: HandleScope,
    handle_data: HandleData,
    os_thread: std::thread::Thread,
    jthread: Handle<Object>,
    class_loader: ObjectPtr,
    interpreter: Interpreter,
    vm: VMPtr,
    heap: HeapPtr,
    lab: LocalAllocBuf,
}

impl Thread {
    fn new(vm: &VM, os_thread: std::thread::Thread) -> Self {
        let mut handle_data = HandleData::new();
        let handle_scope = HandleScope::new_with_data(&mut handle_data);
        let stack_size = vm.cfg.stack_size;
        let stack_addr = vm.heap().alloc_code(stack_size);
        let vm = VMPtr::from_ref(vm);
        let heap = HeapPtr::from_ref(vm.heap());
        let interpreter = Interpreter::new(stack_addr, stack_size, vm);
        return Self {
            _handle_scope: handle_scope,
            handle_data,
            os_thread,
            jthread: Handle::null(),
            class_loader: ObjectPtr::null(),
            interpreter,
            vm,
            heap,
            lab: LocalAllocBuf::default(),
        };
    }

    pub fn current() -> ThreadPtr {
        let thread = VM_THREAD.with(|t| t.get());
        return thread;
    }

    pub fn attach_current_thread(vm: &VM) {
        if Thread::current().is_not_null() {
            return;
        }
        let thread = Box::new(Thread::new(vm, std::thread::current()));
        thread.register_thread_local();
        vm.thread_mgr.add_thread(thread);
    }

    pub fn detach_current_thread() {
        let thread = Thread::current();
        if thread.is_not_null() {
            thread.vm().thread_mgr.remove_thread(thread.thread_id());
            thread.deregister_thread_local();
        }
    }

    pub(crate) fn create_jthread_and_bind(thread: ThreadPtr, thread_group: ObjectPtr) {
        if thread.jthread.is_not_null() {
            return;
        }
        thread
            .vm
            .shared_objs()
            .class_infos()
            .java_lang_thread_info()
            .new_jthread_with_native_id(
                thread.thread_id() as JInt,
                thread_group,
                0,
                1,
                |jthread| {
                    thread.as_mut_ref().jthread = jthread;
                },
                thread,
            );
    }

    pub fn thread_id(&self) -> u64 {
        return self.os_thread.id().as_u64().into();
    }

    pub(crate) fn set_handle_data(&mut self, handle_data: HandleData) {
        self.handle_data = handle_data;
    }

    pub(crate) fn handle_data(&self) -> &HandleData {
        &self.handle_data
    }

    pub(crate) fn handle_data_mut(&mut self) -> &mut HandleData {
        &mut self.handle_data
    }

    pub(crate) fn lab(&self) -> &LocalAllocBuf {
        &self.lab
    }

    pub(crate) fn lab_mut(&mut self) -> &mut LocalAllocBuf {
        &mut self.lab
    }

    pub fn jthread(&self) -> ObjectPtr {
        return self.jthread.as_ptr();
    }

    pub fn vm(&self) -> &VM {
        return self.vm.as_ref();
    }

    pub fn vm_ptr(&self) -> VMPtr {
        return self.vm;
    }

    pub(crate) fn heap(&self) -> &Heap {
        return self.heap.as_ref();
    }

    pub fn class_loader(&self) -> ObjectPtr {
        return self.class_loader;
    }

    pub(crate) fn interpreter(&self) -> &Interpreter {
        &self.interpreter
    }

    pub(crate) fn interpreter_mut(&mut self) -> &mut Interpreter {
        &mut self.interpreter
    }

    fn register_thread_local(&self) {
        VM_THREAD.with(|t| {
            t.set(ThreadPtr::from_ref(self));
        });
    }

    fn deregister_thread_local(&self) {
        VM_THREAD.with(|t| {
            t.set(ThreadPtr::null());
        });
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        log::trace!("Thread::Drop {}", self.thread_id());
    }
}
