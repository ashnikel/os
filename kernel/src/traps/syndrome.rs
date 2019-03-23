#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Fault {
    AddressSize,
    Translation,
    AccessFlag,
    Permission,
    Alignment,
    TlbConflict,
    Other(u8),
}

impl From<u32> for Fault {
    fn from(val: u32) -> Fault {
        use self::Fault::*;

        match (val & 0xFF) as u8 {
            0b000_000...0b000_011 => AddressSize,
            0b000_100...0b000_111 => Translation,
            0b001_001...0b001_011 => AccessFlag,
            0b001_101...0b001_111 => Permission,
            0b100_001 => Alignment,
            0b110_000 => TlbConflict,
            other => Other(other),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Syndrome {
    Unknown,
    WfiWfe,
    McrMrc,
    McrrMrrc,
    LdcStc,
    SimdFp,
    Vmrs,
    Mrrc,
    IllegalExecutionState,
    Svc(u16),
    Hvc(u16),
    Smc(u16),
    MsrMrsSystem,
    InstructionAbort { kind: Fault, level: u8 },
    PCAlignmentFault,
    DataAbort { kind: Fault, level: u8 },
    SpAlignmentFault,
    TrappedFpu,
    SError,
    Breakpoint,
    Step,
    Watchpoint,
    Brk(u16),
    Other(u32),
}

/// Converts a raw syndrome value (ESR) into a `Syndrome` (ref: D1.10.4).
impl From<u32> for Syndrome {
    fn from(esr: u32) -> Syndrome {
        use self::Syndrome::*;

        let exception_class = esr >> 26;
        match exception_class {
            0b000_000 => Unknown,
            0b000_001 => WfiWfe,
            0b000_011 => McrMrc,
            0b000_100 => McrrMrrc,
            0b000_101 => McrMrc,
            0b000_110 => LdcStc,
            0b000_111 => SimdFp,
            0b001_000 => Vmrs,
            0b001_100 => Mrrc,
            0b001_110 => IllegalExecutionState,
            0b010_001 => Svc((esr & 0xFFFF) as u16), // AArch32
            0b010_010 => Hvc((esr & 0xFFFF) as u16), // AArch32
            0b010_011 => Smc((esr & 0xFFFF) as u16), // AArch32
            0b010_101 => Svc((esr & 0xFFFF) as u16), // AArch64
            0b010_111 => Smc((esr & 0xFFFF) as u16), // AArch64
            0b010_110 => Hvc((esr & 0xFFFF) as u16), // AArch64
            0b011_000 => MsrMrsSystem,
            0b100_000 => InstructionAbort {
                kind: Fault::from(esr),
                level: ((esr & 0b11) as u8),
            }, // from a lower EL
            0b100_001 => InstructionAbort {
                kind: Fault::from(esr),
                level: ((esr & 0b11) as u8),
            }, // same EL
            0b100_010 => PCAlignmentFault,
            0b100_100 => DataAbort {
                kind: Fault::from(esr),
                level: ((esr & 0b11) as u8),
            }, // from a lower EL
            0b100_101 => DataAbort {
                kind: Fault::from(esr),
                level: ((esr & 0b11) as u8),
            }, // same EL
            0b100_110 => SpAlignmentFault,
            0b101_000 => TrappedFpu, // AArch32
            0b101_100 => TrappedFpu, // AArch64
            0b101_111 => SError,
            0b110_000 => Breakpoint,                 // from a lower EL
            0b110_001 => Breakpoint,                 // same EL
            0b110_010 => Step,                       // from a lower EL
            0b110_011 => Step,                       // same EL
            0b110_100 => Watchpoint,                 // from a lower EL
            0b110_101 => Watchpoint,                 // same EL
            0b111_000 => Brk((esr & 0xFFFF) as u16), // AArch32
            0b111_100 => Brk((esr & 0xFFFF) as u16), // AArch64
            other => Other(other),
        }
    }
}
