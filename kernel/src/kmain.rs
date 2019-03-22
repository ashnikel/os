#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(optin_builtin_traits)]
#![feature(decl_macro)]
#![feature(repr_align)]
#![feature(attr_literals)]
#![feature(exclusive_range_pattern)]
#![feature(i128_type)]
#![feature(never_type)]
#![feature(unique)]
#![feature(pointer_methods)]
#![feature(naked_functions)]
#![feature(fn_must_use)]
#![feature(alloc, allocator_api, global_allocator)]

#[macro_use]
#[allow(unused_imports)]
extern crate alloc;
extern crate fat32;
extern crate pi;
extern crate stack_vec;

pub mod aarch64;
pub mod allocator;
pub mod console;
pub mod fs;
pub mod lang_items;
pub mod mutex;
pub mod process;
pub mod shell;
pub mod traps;
pub mod vm;

#[cfg(not(test))]
use allocator::Allocator;
use console::kprint;
use console::kprintln;
use fat32::traits::BlockDevice;
use fs::sd::Sd;
use fs::FileSystem;
use process::GlobalScheduler;

#[cfg(not(test))]
#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();

pub static FILE_SYSTEM: FileSystem = FileSystem::uninitialized();

pub static SCHEDULER: GlobalScheduler = GlobalScheduler::uninitialized();

pub fn print_atags() {
    kprintln!("ATAGS:\n");
    for atag in pi::atags::Atags::get() {
        kprintln!("{:#?}", atag);
    }
    kprintln!();
}

pub fn check_alloc() {
    let mut v = vec![];
    for i in 0..100 {
        v.push(i);
        kprintln!("{:?}", v);
    }
    let hello_string = String::from("Hello String!");
    kprintln!("{}", hello_string);
}

pub fn check_mbr() {
    kprint!("Master Boot Record:");
    let mut sd = Sd::new().unwrap();
    let mut buf = [0u8; 512];
    let _read = sd.read_sector(0, &mut buf);

    let mut i = 0;
    for byte in buf.iter() {
        if i % 20 == 0 {
            kprintln!();
            kprint!("offset {:03X}: ", i);
        }
        kprint!("{:02X} ", byte);
        i += 1;
    }
    kprintln!();
}

pub fn check_root_dir() {
    use fat32::traits::{Dir, Entry, FileSystem};

    kprintln!("Root directory contents:");
    for entry in FILE_SYSTEM.open_dir("/").unwrap().entries().unwrap() {
        kprintln!("{}", entry.name());
    }
}

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn kmain() {
    pi::timer::spin_sleep_ms(1000);

    // print_atags();

    ALLOCATOR.initialize();
    FILE_SYSTEM.initialize();

    // check_root_dir();
    // check_alloc();
    // check_mbr();

    kprintln!("Well, hello...");
    unsafe {
        let el = aarch64::current_el();
        kprintln!("Current Exception Level is EL{}", el);
    }
    unsafe { asm!("brk 2" :::: "volatile"); }
    shell::shell("> ");
}
