# libargs
On most Rust platforms, `std::env::args` will work in a function called from a C program. However, this is not the case on Linux. The following platforms are supported:

* `cfg(all(target_os = "linux", target_env = "gnu"))`: glibc will pass command line arguments to static constructors, which is non-standard
* `cfg(all(target_os = "linux", not(target_env = "gnu")))`: The `envp` pointer passed to `_start` is stored in the `__environ` symbol. As command line arguments precede it in the stack, we can walk the stack backwards from `__environ` to find `argc` and `argv`. This works on all platforms I've tested it on, however it takes linear time with regards to argc.
* `cfg(not(target_os = "linux"))`: `std::env::args` is used, which works on most platforms (including Windows and macOS). On unsupported platforms, an empty Vec will be returned.
