use cpu::z80::inst::{Inst, Src, Dest};
use cpu::z80::regs::{Reg8, Reg16};

pub type InstSize = usize;
pub type InstTime = usize;

pub struct InstProps {
    pub size: InstSize,
    pub time: InstTime,
}

impl InstProps {
    pub fn from_inst(inst: &Inst) -> InstProps {
        match inst {
            Inst::NOP => InstProps { size: 1, time: 4 },
            Inst::LD16(Dest::Reg(Reg16::BC), Src::Liter(_)) => InstProps { size: 3, time: 10 },
            Inst::LD8(Dest::IndReg(Reg16::BC), Src::Reg(Reg8::A)) => InstProps { size: 1, time: 7 },
            _ => unimplemented!("props of given instruction is not implemented"),
        }
    }
}