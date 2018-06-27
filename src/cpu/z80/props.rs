use cpu::z80::inst::{Inst, Src, Dest};

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
            Inst::ADD16(Dest::Reg(_), Src::Reg(_)) => InstProps { size: 1, time: 11 },
            Inst::DEC8(_) => InstProps { size: 1, time: 4 },
            Inst::DEC16(_) => InstProps { size: 1, time: 6 },
            Inst::INC8(_) => InstProps { size: 1, time: 4 },
            Inst::INC16(_) => InstProps { size: 1, time: 6 },
            Inst::LD16(Dest::Reg(_), Src::Liter(_)) => InstProps { size: 3, time: 10 },
            Inst::LD8(Dest::Reg(_), Src::Liter(_)) => InstProps { size: 2, time: 7 },
            Inst::LD8(Dest::Reg(_), Src::IndReg(_)) => InstProps { size: 1, time: 7 },
            Inst::LD8(Dest::IndReg(_), Src::Reg(_)) => InstProps { size: 1, time: 7 },
            Inst::RLCA => InstProps { size: 1, time: 4 },
            Inst::RRCA => InstProps { size: 1, time: 4 },
            _ => unimplemented!("props of given instruction is not implemented"),
        }
    }
}