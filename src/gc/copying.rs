use crate::memory::space::SemiSpace;
use crate::memory::Address;

pub struct CopyingCollector {
    age_mark: Address,
    age_threshhold: u32,
}

impl CopyingCollector {
    //
}
