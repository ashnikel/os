use console::kprintln;
use pi::interrupt::Interrupt;
use pi::timer::tick_in;
use process::TICK;
use traps::TrapFrame;

pub fn handle_irq(interrupt: Interrupt, tf: &mut TrapFrame) {
    if interrupt == Interrupt::Timer1 {
        kprintln!("[tick]");
        tick_in(TICK);
    }
}
