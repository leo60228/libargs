//! See documentation on `args`.

#[cfg(all(
    target_os = "linux",
    any(not(target_env = "gnu"), feature = "force_walk")
))]
mod imp {
    use once_cell::sync::Lazy;
    use std::ffi::CStr;
    use std::os::raw::*;

    extern "C" {
        pub static __environ: *const *const c_char;
    }

    fn raw_args() -> (c_int, *const *const c_char) {
        let mut walk_environ = unsafe { __environ as *const usize };
        walk_environ = walk_environ.wrapping_offset(-1);
        let mut i = 0;

        loop {
            let argc_ptr = walk_environ.wrapping_offset(-1) as *const c_int;
            let argc = unsafe { *argc_ptr };
            if argc == i {
                break (argc, walk_environ as *const *const c_char);
            }
            walk_environ = walk_environ.wrapping_offset(-1);
            i += 1;
        }
    }

    /// Get the arguments as a vector, even if the function is called from a C program via FFI.
    /// Supports Windows, macOS, and Linux, and will return an empty Vec on unsupported platforms.
    pub fn args() -> Vec<String> {
        static ARGS: Lazy<Vec<String>> = Lazy::new(|| {
            let (argc, argv) = raw_args();
            (0..argc)
                .map(|i| unsafe {
                    CStr::from_ptr(*argv.wrapping_offset(i as isize))
                        .to_string_lossy()
                        .into()
                })
                .collect()
        });
        ARGS.clone()
    }
}

#[cfg(all(
    target_os = "linux",
    all(target_env = "gnu", not(feature = "force_walk"))
))]
mod imp {
    use once_cell::sync::OnceCell;
    use std::ffi::CStr;
    use std::os::raw::*;

    static ARGS: OnceCell<Vec<String>> = OnceCell::new();

    #[used]
    #[link_section = ".init_array.00099"]
    #[no_mangle]
    static SET_ARGS: [extern "C" fn(c_int, *const *const c_char, *const *const c_char); 1] = {
        extern "C" fn set_args(
            argc: c_int,
            argv: *const *const c_char,
            _env: *const *const c_char,
        ) {
            ARGS.set(
                (0..argc)
                    .map(|i| unsafe {
                        let cstr = CStr::from_ptr(*argv.offset(i as isize));
                        String::from_utf8_lossy(cstr.to_bytes()).into()
                    })
                    .collect(),
            )
            .unwrap();
        }

        [set_args]
    };

    /// Get the arguments as a vector, even if the function is called from a C program via FFI.
    /// Supports Windows, macOS, and Linux, and will return an empty Vec on unsupported platforms.
    pub fn args() -> Vec<String> {
        ARGS.get().unwrap().clone()
    }
}

#[cfg(not(target_os = "linux"))]
mod imp {
    /// Get the arguments as a vector, even if the function is called from a C program via FFI.
    /// Supports Windows, macOS, and Linux, and will return an empty Vec on unsupported platforms.
    pub fn args() -> Vec<String> {
        std::env::args().collect()
    }
}

pub use imp::args;

#[cfg(test)]
mod tests {
    #[test]
    fn args() {
        assert_eq!(super::args(), std::env::args().collect::<Vec<_>>());
    }
}
