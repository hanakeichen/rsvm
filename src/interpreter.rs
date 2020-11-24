use crate::memory::Address;

pub trait Interpreter {
    fn execute(code: Address);
}
