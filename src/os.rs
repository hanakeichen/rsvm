use std::ptr::null_mut;

use crate::memory::{is_align_of, Address};

static mut PAGE_SIZE: isize = -1;

pub fn init() {
    #[cfg(target_family = "unix")]
    unsafe {
        PAGE_SIZE = libc::sysconf(libc::_SC_PAGESIZE) as isize;
    }
    #[cfg(target_os = "windows")]
    {
        use winapi::um::sysinfoapi::GetSystemInfo;
        use winapi::um::sysinfoapi::{LPSYSTEM_INFO, SYSTEM_INFO};

        unsafe {
            let mut sys_info: SYSTEM_INFO = std::mem::zeroed();
            GetSystemInfo(&mut sys_info as LPSYSTEM_INFO);
            PAGE_SIZE = sys_info.dwPageSize as isize;
        }
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
    debug_assert!(is_align_of(size, page_size()));
    #[cfg(target_family = "unix")]
    {
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
    #[cfg(target_os = "windows")]
    {
        use winapi::um::{
            memoryapi::VirtualAlloc,
            winnt::{MEM_RESERVE, PAGE_NOACCESS},
        };

        let res = unsafe { VirtualAlloc(null_mut(), size, MEM_RESERVE, PAGE_NOACCESS) };
        if res.is_null() {
            return Address::null();
        }
        return Address::new(res.cast());
    }
}

pub fn commit_memory(addr: Address, size: usize, exec: bool) -> bool {
    debug_assert!(is_align_of(size, page_size()));
    #[cfg(target_family = "unix")]
    {
        let mut prot = libc::PROT_READ | libc::PROT_WRITE;
        if exec {
            prot |= libc::PROT_EXEC;
        }
        let res = unsafe {
            libc::mmap(
                addr.raw_ptr() as _,
                size,
                prot,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };
        return res != libc::MAP_FAILED;
    }
    #[cfg(target_os = "windows")]
    {
        use winapi::um::{
            memoryapi::VirtualAlloc,
            winnt::{MEM_COMMIT, PAGE_EXECUTE_READWRITE, PAGE_READWRITE},
        };

        let prot = if !exec {
            PAGE_READWRITE
        } else {
            PAGE_EXECUTE_READWRITE
        };

        let res = unsafe { VirtualAlloc(addr.raw_ptr() as _, size, MEM_COMMIT, prot) };
        return !res.is_null();
    }
}

pub fn release_memory(addr: Address, size: usize) -> i32 {
    #[cfg(target_family = "unix")]
    {
        unsafe {
            return libc::munmap(addr.raw_ptr() as _, size);
        }
    }
    #[cfg(target_os = "windows")]
    {
        use winapi::um::{memoryapi::VirtualFree, winnt::MEM_RELEASE};

        return unsafe { VirtualFree(addr.raw_ptr() as _, size, MEM_RELEASE) };
    }
}
