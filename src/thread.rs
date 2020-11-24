use crate::handle::HandleData;
use crate::memory::Address;
use std::cell::RefCell;
use std::sync::Arc;

thread_local! {
    static VM_THREAD: RefCell<Option<Thread>> = RefCell::new(None);
}

pub struct ThreadManager {
    threads: Vec<Thread>,
}

#[derive(Clone)]
pub struct Thread {
    thread_impl: Arc<ThreadImpl>,
}

impl Thread {
    pub fn current() -> Thread {
        VM_THREAD.with(|t| {
            t.borrow_mut()
                .as_mut()
                .expect("Thread::current(): cannot find current thread")
                .clone()
        })
    }

    pub fn set_handle_data(&mut self, handle_data: HandleData) {
        unsafe { Arc::get_mut_unchecked(&mut self.thread_impl).handle_data = handle_data }
    }

    pub fn handle_data(&self) -> &HandleData {
        &self.thread_impl.handle_data
    }

    pub fn handle_data_mut(&mut self) -> &mut HandleData {
        unsafe { &mut Arc::get_mut_unchecked(&mut self.thread_impl).handle_data }
    }

    // pub fn main() -> Rc<Thread> {}
}

struct ThreadImpl {
    handle_data: HandleData,
    os_thread: std::thread::Thread,
    stack_size: usize,
    pc: Address,
}
