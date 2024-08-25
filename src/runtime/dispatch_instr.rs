#[macro_export]
macro_rules! goto_label {
    ( $label:expr ) => {
        unsafe {
            use core::arch::asm;

            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            {
                asm!(concat!("jmp ", $label));
            }
            #[cfg(any(target_arch = "aarch64"))]
            {
                asm!(concat!("b ", $label));
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "aarch64", target_arch = "x86_64")))]
            {
                compile_error!("goto_label not implemented");
            }
        }
    };
}

#[macro_export]
macro_rules! goto_label_addr {
    ( $label_addr:expr ) => {
        unsafe {
            use core::arch::asm;

            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            {
                asm!("jmp {}", in(reg) $label_addr);
            }
            #[cfg(any(target_arch = "aarch64"))]
            {
                asm!("br {}", in(reg) $label_addr);
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "aarch64", target_arch = "x86_64")))]
            {
                compile_error!("goto_label_addr not implemented");
            }
        }
    };
}

#[macro_export]
macro_rules! label {
    ( $label:expr ) => {
        unsafe {
            use core::arch::asm;

            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            #[allow(named_asm_labels)]
            {
                asm!(concat!($label, ":"));
            }

            #[cfg(target_arch = "aarch64")]
            #[allow(named_asm_labels)]
            {
                asm!(concat!($label, ":"), out("x8") _);
            }

            #[cfg(not(any(target_arch = "x86", target_arch = "aarch64", target_arch = "x86_64")))]
            {
                compile_error!("label not implemented");
            }
        }
    };
}

#[macro_export]
macro_rules! label_addr {
    ( $label_name:expr) => {
        unsafe {
            use core::arch::asm;

            let mut _x: u64 = 0;

            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            {
                asm!(concat!("lea {:r}, [rip +", $label_name, "]"), out(reg) _x);
            }
            #[cfg(any(target_arch = "aarch64"))]
            {
                asm!(
                    concat!("adrp {tmp}, ", $label_name, "@PAGE"),
                    concat!("add {tmp}, {tmp}, ", $label_name, "@PAGEOFF"),
                    tmp = out(reg) _x,
                );
            }

            #[cfg(not(any(target_arch = "x86", target_arch = "aarch64", target_arch = "x86_64")))]
            {
                compile_error!("goto_label_addr not implemented");
            }
            _x
        }
    };
}

#[macro_export]
macro_rules! reserve_value {
    ($val:expr) => {
        unsafe {
            use core::arch::asm;

            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            {
                asm!("mov rdi, {0}", in(reg) $val, out("rdi") _);
            }

            #[cfg(any(target_arch = "aarch64"))]
            {
                asm!("mov x0, {0}", in(reg) $val, out("x0") _);
            }

            #[cfg(not(any(target_arch = "x86", target_arch = "aarch64", target_arch = "x86_64")))]
            {
                compile_error!("reserve_value not implemented");
            }
        }
    };
}

#[macro_export]
macro_rules! load_reserved_value {
    () => {
        {
            use core::arch::asm;

            let val: u64;

            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            {
                asm!("mov {}, rdi", out(reg) val);
            }

            #[cfg(any(target_arch = "aarch64"))]
            {
                asm!("mov {}, x0", out(reg) val);
            }

            #[cfg(not(any(target_arch = "x86", target_arch = "aarch64", target_arch = "x86_64")))]
            {
                compile_error!("load_reserved_value not implemented");
            }

            val
        }
    };
}

// #[macro_export]
// macro_rules! reset_sp_offset_register {
//     () => {
//         unsafe {
//             use core::arch::asm;

//             #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
//             {
//                 // asm!("mov {}, rdi", out(reg) val);
//             }

//             #[cfg(any(target_arch = "aarch64"))]
//             {
//                 asm!("nop", out("x8") _);
//             }

//             #[cfg(not(any(target_arch = "x86", target_arch = "aarch64", target_arch = "x86_64")))]
//             {
//                 compile_error!("load_reserved_value not implemented");
//             }

//         }
//     };
// }

#[cfg(test)]
mod tests {
    fn get_label_addr() -> u64 {
        return label_addr!("next");
    }

    #[inline(always)]
    fn goto_label_addr(label: u64) {
        goto_label_addr!(label);
    }

    #[test]
    fn goto_test() {
        let next: u64 = get_label_addr();
        assert_ne!(next, 0);
        goto_label_addr(next);

        assert!(false);

        label!("next");
        goto_label!("last");

        assert!(false);

        label!("last");

        assert!(true);
    }
}
