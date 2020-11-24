use crate::memory::Address;

static mut PAGE_SIZE: isize = -1;

pub fn init() {
    unsafe {
        PAGE_SIZE = libc::sysconf(libc::_SC_PAGESIZE) as isize;
    }
}

pub fn page_size() -> usize {
    unsafe {
        if PAGE_SIZE == -1 {
            panic!("must call os::init() prior to using it");
        }
        return PAGE_SIZE as usize;
    }
}

pub fn reserve_memory(size: usize) -> Address {
    let res = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        )
    };
    if res == libc::MAP_FAILED {
        return Address::null();
    } else {
        return Address::new(res.cast());
    }
}

pub fn commit_memory(addr: Address, size: usize, exec: bool) -> bool {
    let mut prot = libc::PROT_READ | libc::PROT_WRITE;
    if exec {
        prot |= libc::PROT_EXEC;
    }
    let res = unsafe {
        libc::mmap(
            addr.ptr() as _,
            size,
            prot,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_FIXED,
            -1,
            0,
        )
    };
    return res != libc::MAP_FAILED;
}
