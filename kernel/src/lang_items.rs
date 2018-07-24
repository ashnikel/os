use console::kprintln;

#[no_mangle]
#[cfg(not(test))]
#[lang = "panic_fmt"]
pub extern fn panic_fmt(fmt: ::std::fmt::Arguments, file: &'static str, line: u32, col: u32) -> ! {
    kprintln!(" __________");
    kprintln!("( OH SHI-- )");
    kprintln!(" ----------");
    kprintln!("        o   ^__^");
    kprintln!("         o  (oo)\\_______");
    kprintln!("            (__)\\       )\\/\\");
    kprintln!("                ||----w |");
    kprintln!("                ||     ||");
    kprintln!();
    kprintln!("       *** PANIC ***");
    kprintln!();
    kprintln!("paniced: {}", fmt);
    kprintln!("    --> {}:{}:{}", file, line, col);


    loop { unsafe { asm!("wfe") } }
}

#[cfg(not(test))] #[lang = "eh_personality"] pub extern fn eh_personality() {}
