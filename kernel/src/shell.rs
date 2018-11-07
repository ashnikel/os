use console::{kprint, kprintln, CONSOLE};
use fat32::traits::{Dir, Entry, FileSystem};
use FILE_SYSTEM;
use stack_vec::StackVec;
use std::path::{Component, Path, PathBuf};

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs
}

/// A structure representing a single shell command.
struct Command<'a> {
    args: StackVec<'a, &'a str>
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        self.args[0]
    }

    fn exec(&self, cwd: &mut PathBuf) {
        match self.path() {
                    "cd" => cmd_cd(&self.args[1..], cwd),
                    "echo" => cmd_echo(&self.args[1..]),
                    "pwd" =>  cmd_pwd(&self.args[1..], cwd),
                    "reset" => {
                        kprintln!("goodbye!");
                        kprintln!("press `<ctrl-a>`, `k` to exit");
                        jump_to(BOOTLOADER_START);
                    }
                    "panic" => {
                        panic!("oh dear!");
                    }
                    _ => kprintln!("unknown command: {}", self.path()),
        }
    }
}

pub fn cmd_cd(args: &[&str], cwd: &mut PathBuf) {
    if args.len() == 0 {
        cwd.push(PathBuf::from("/"));
        return;
    }
    if args.len() > 1 {
        kprintln!("cd: too many arguments");
        return;
    }
    match args[0] {
        "." => {},
        ".." => { cwd.pop(); },
        path => {
            let mut new_cwd = cwd.clone();
            new_cwd.push(path);
            let norm = path_normalize(&new_cwd);
            if let Err(_) = FILE_SYSTEM.open_dir(&norm) {
                kprintln!("cd: no such directory");
                return;
            }
            cwd.push(norm);
        }
    }
}

pub fn path_normalize(path: &PathBuf) -> PathBuf {
    let mut norm = PathBuf::new();
    for component in path.components() {
        match component {
            Component::RootDir | Component::Normal(_) => norm.push(component.as_os_str()),
            Component::ParentDir => {norm.pop();},
            _ => {}
        }
    }
    norm
}

pub fn cmd_echo(args: &[&str]) {
    for arg in args.iter() {
        kprint!("{} ", arg);
    }
    kprintln!();
}

pub fn cmd_pwd(args: &[&str], cwd: &PathBuf) {
    if args.len() > 0 {
        kprintln!("pwd: too many arguments");
        return;
    }
    kprintln!("{}", cwd.display());
}

const BS: u8 = 0x08;
const BEL: u8 = 0x07;
const LF: u8 = 0x0A;
const CR: u8 = 0x0D;
const DEL: u8 = 0x7F;

fn read_line(mut line_vec: StackVec<u8>) -> &str {
    loop {
        let byte = CONSOLE.lock().read_byte();
        match byte {
            // Printable characters
            byte @ 0x20 ... 0x7E => {
                match line_vec.push(byte) {
                    Ok(()) => kprint!("{}", byte as char),
                    Err(()) => kprint!("{}", BEL as char),
                }
            }
            BS | DEL => {
                match line_vec.pop() {
                    Some(_) => {
                        kprint!("{}", BS as char);
                        kprint!(" ");
                        kprint!("{}", BS as char);
                    }
                    None => kprint!("{}", BEL as char),
                }
            }
            CR | LF => {
                kprintln!();
                break;
            }
            _ => {
                kprint!("{}", BEL as char);
            }
        }
    }

    ::std::str::from_utf8(line_vec.into_slice()).unwrap()
}

/// Branches to the address `addr` unconditionally.
fn jump_to(addr: *mut u8) -> ! {
    unsafe {
        asm!("br $0" : : "r"(addr as usize));
        loop { asm!("nop" :::: "volatile")  }
    }
}

const MAXBUF: usize = 512;
const MAXARGS: usize = 64;

const BOOTLOADER_START_ADDR: usize = 0x4000000;
const BOOTLOADER_START: *mut u8 = BOOTLOADER_START_ADDR as *mut u8;
/// Starts a shell using `prefix` as the prefix for each line. This function
/// never returns: it is perpetually in a shell loop.
pub fn shell(prefix: &str) -> ! {
    let mut cwd = PathBuf::from("/");

    loop {
        kprint!("{}{}", cwd.display(), prefix);
        let mut buf = [0u8; MAXBUF];
        let line_vec = StackVec::new(&mut buf);
        let line = read_line(line_vec);
        match Command::parse(line, &mut [""; MAXARGS]) {
            Ok(cmd) => {
                cmd.exec(&mut cwd);
            }
            Err(Error::TooManyArgs) => kprintln!("error: too many arguments"),
            Err(Error::Empty) => { }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_path_normalize() {
        let path = PathBuf::from("/1/2/../3/./4/../../5/");
        assert_eq!(PathBuf::from("/1/5"), path_normalize(&path));

        let path = PathBuf::from("/../../.././.");
        assert_eq!(PathBuf::from("/"), path_normalize(&path));
    }
}