use super::{status, CPU};

pub enum Condition {
    Always,
    CarryClear,
    CarrySet,
    Equal,
    Minus,
    NotEqual,
    Plus,
    OverflowClear,
    OverflowSet,
}

impl Condition {
    pub fn eval(&self, cpu: &CPU) -> bool {
        match self {
            Condition::Always => true,
            Condition::CarryClear => cpu.regs.status_flag_is_clear(status::Flag::C),
            Condition::CarrySet => cpu.regs.status_flag_is_set(status::Flag::C),
            Condition::Equal => cpu.regs.status_flag_is_set(status::Flag::Z),
            Condition::Minus => cpu.regs.status_flag_is_set(status::Flag::N),
            Condition::NotEqual => cpu.regs.status_flag_is_clear(status::Flag::Z),
            Condition::Plus => cpu.regs.status_flag_is_clear(status::Flag::N),
            Condition::OverflowClear => cpu.regs.status_flag_is_clear(status::Flag::V),
            Condition::OverflowSet => cpu.regs.status_flag_is_set(status::Flag::V),
        }
    }

    pub fn branch_mnemo(&self) -> &'static str {
        match self {
            Condition::Always => "BRA",
            Condition::CarryClear => "BCC",
            Condition::CarrySet => "BCS",
            Condition::Equal => "BEQ",
            Condition::Minus => "BMI",
            Condition::NotEqual => "BNE",
            Condition::Plus => "BPL",
            Condition::OverflowClear => "BVC",
            Condition::OverflowSet => "BVS",            
        }
    }
}