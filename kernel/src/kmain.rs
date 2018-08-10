#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(optin_builtin_traits)]
#![feature(decl_macro)]
#![feature(repr_align)]
#![feature(attr_literals)]
#![feature(exclusive_range_pattern)]
#![feature(alloc, allocator_api, global_allocator)]
#![feature(pointer_methods)]

#[macro_use]
#[allow(unused_imports)]
extern crate alloc;
extern crate pi;
extern crate stack_vec;
extern crate fat32;

pub mod allocator;
pub mod lang_items;
pub mod mutex;
pub mod console;
pub mod shell;
pub mod fs;

#[cfg(not(test))]
use allocator::Allocator;
use console::kprintln;
use fs::FileSystem;

#[cfg(not(test))]
#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();

pub static FILE_SYSTEM: FileSystem = FileSystem::uninitialized();

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

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn kmain() {
    pi::timer::spin_sleep_ms(1000);

    print_atags();

    kprintln!("Well, hello...");

    ALLOCATOR.initialize();
    check_alloc();

    shell::shell("> ");
}
