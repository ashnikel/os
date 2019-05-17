use std::collections::VecDeque;

use mutex::Mutex;
use process::{Id, Process, State};
use shell;
use traps::TrapFrame;

/// The `tick` time.
// FIXME: When you're ready, change this to something more reasonable.
pub const TICK: u32 = 2 * 1000 * 1000;

/// Process scheduler for the entire machine.
#[derive(Debug)]
pub struct GlobalScheduler(Mutex<Option<Scheduler>>);

impl GlobalScheduler {
    /// Returns an uninitialized wrapper around a local scheduler.
    pub const fn uninitialized() -> GlobalScheduler {
        GlobalScheduler(Mutex::new(None))
    }

    /// Adds a process to the scheduler's queue and returns that process's ID.
    /// For more details, see the documentation on `Scheduler::add()`.
    pub fn add(&self, process: Process) -> Option<Id> {
        self.0
            .lock()
            .as_mut()
            .expect("scheduler uninitialized")
            .add(process)
    }

    /// Performs a context switch using `tf` by setting the state of the current
    /// process to `new_state`, saving `tf` into the current process, and
    /// restoring the next process's trap frame into `tf`. For more details, see
    /// the documentation on `Scheduler::switch()`.
    #[must_use]
    pub fn switch(&self, new_state: State, tf: &mut TrapFrame) -> Option<Id> {
        self.0
            .lock()
            .as_mut()
            .expect("scheduler uninitialized")
            .switch(new_state, tf)
    }

    /// Initializes the scheduler and starts executing processes in user space
    /// using timer interrupt based preemptive scheduling. This method should
    /// not return under normal conditions.
    pub fn start(&self) {
        let mut process = Process::new().unwrap();
        process.trap_frame.elr = run_shell as u64;
        process.trap_frame.sp = process.stack.top().as_u64();
        process.trap_frame.spsr = 0;
        let tf = &*process.trap_frame;

        unsafe {
            asm!("mov sp, $0
                  bl context_restore
                  adr x0, _start
                  mov sp, x0
                  mov x0, xzr
                  mov lr, xzr
                  eret"
                :: "r"(tf)
                :: "volatile");
        }
    }
}

#[derive(Debug)]
struct Scheduler {
    processes: VecDeque<Process>,
    current: Option<Id>,
    last_id: Option<Id>,
}

impl Scheduler {
    /// Returns a new `Scheduler` with an empty queue.
    fn new() -> Scheduler {
        unimplemented!("Scheduler::new()")
    }

    /// Adds a process to the scheduler's queue and returns that process's ID if
    /// a new process can be scheduled. The process ID is newly allocated for
    /// the process and saved in its `trap_frame`. If no further processes can
    /// be scheduled, returns `None`.
    ///
    /// If this is the first process added, it is marked as the current process.
    /// It is the caller's responsibility to ensure that the first time `switch`
    /// is called, that process is executing on the CPU.
    fn add(&mut self, mut process: Process) -> Option<Id> {
        unimplemented!("Scheduler::add()")
    }

    /// Sets the current process's state to `new_state`, finds the next process
    /// to switch to, and performs the context switch on `tf` by saving `tf`
    /// into the current process and restoring the next process's trap frame
    /// into `tf`. If there is no current process, returns `None`. Otherwise,
    /// returns `Some` of the process ID that was context switched into `tf`.
    ///
    /// This method blocks until there is a process to switch to, conserving
    /// energy as much as possible in the interim.
    fn switch(&mut self, new_state: State, tf: &mut TrapFrame) -> Option<Id> {
        unimplemented!("Scheduler::switch()")
    }
}

extern "C" fn run_shell() {
    unsafe {
        asm!("brk 1" :::: "volatile");
    }
    unsafe {
        asm!("brk 2" :::: "volatile");
    }
    shell::shell("user0> ");
    unsafe {
        asm!("brk 3" :::: "volatile");
    }
    loop {
        shell::shell("user1> ");
    }
}
