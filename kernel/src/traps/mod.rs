mod irq;
mod syndrome;
mod syscall;
mod trap_frame;

use pi::interrupt::{Controller, Interrupt};

pub use self::trap_frame::TrapFrame;

use self::irq::handle_irq;
use self::syndrome::Syndrome;
use self::syscall::handle_syscall;
use aarch64;
use console::kprintln;
use shell;

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Kind {
    Synchronous = 0,
    Irq = 1,
    Fiq = 2,
    SError = 3,
}

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Source {
    CurrentSpEl0 = 0,
    CurrentSpElx = 1,
    LowerAArch64 = 2,
    LowerAArch32 = 3,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Info {
    source: Source,
    kind: Kind,
}

/// This function is called when an exception occurs. The `info` parameter
/// specifies the source and kind of exception that has occurred. The `esr` is
/// the value of the exception syndrome register. Finally, `tf` is a pointer to
/// the trap frame for the exception.
#[no_mangle]
pub extern "C" fn handle_exception(info: Info, esr: u32, tf: &mut TrapFrame) {
    kprintln!("Exception: {:?}", info);
    if info.kind == Kind::Synchronous {
        kprintln!("Syndrome: {:?}", Syndrome::from(esr));
    }

    if info.kind == Kind::Synchronous {
        let syndrome = Syndrome::from(esr);
        match syndrome {
            Syndrome::Brk(n) => {
                shell::shell("brk> ");
                tf.elr += 4;
            }
            _ => (),
        }
    } else if info.kind == Kind::Irq {
        let controller = Controller::new();
        let ints = [
            Interrupt::Timer1,
            Interrupt::Timer3,
            Interrupt::Usb,
            Interrupt::Gpio0,
            Interrupt::Gpio1,
            Interrupt::Gpio2,
            Interrupt::Gpio3,
            Interrupt::Uart,
        ];
        for int in ints.iter() {
            if controller.is_pending(*int) {
                handle_irq(*int, tf);
            }
        }
    }
}
