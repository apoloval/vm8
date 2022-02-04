use crate::cpu::z81::bus::Bus;
use crate::cpu::z81::reg::Registers;

pub struct Context<'a, B: Bus> {
    pub bus: &'a mut B,
    pub regs: &'a mut Registers,
}

impl<'a, B: Bus> Context<'a, B> {
    pub fn from(bus: &'a mut B, regs: &'a mut Registers) -> Self {
        Self { bus, regs }
    }
}

pub trait SrcOp<T> {
    fn get<B: Bus>(&self, ctx: &Context<B>) -> T;
}

pub trait DestOp<T> : SrcOp<T> {
    fn set<B: Bus>(&self, ctx: &mut Context<B>, val: T);
}

pub enum Reg8 { A, B, C, D, E, H, L }

impl SrcOp<u8> for Reg8 {
    fn get<B: Bus>(&self, ctx: &Context<B>) -> u8 { 
        match self {
            Reg8::A => ctx.regs.a(),
            Reg8::B => ctx.regs.b(),
            Reg8::C => ctx.regs.c(),
            Reg8::D => ctx.regs.d(),
            Reg8::E => ctx.regs.e(),
            Reg8::H => ctx.regs.h(),
            Reg8::L => ctx.regs.l(),
        }
    }
}

impl DestOp<u8> for Reg8 {
    fn set<B: Bus>(&self, ctx: &mut Context<B>, val: u8) {
        match self {
            Reg8::A => ctx.regs.set_a(val),
            Reg8::B => ctx.regs.set_b(val),
            Reg8::C => ctx.regs.set_c(val),
            Reg8::D => ctx.regs.set_d(val),
            Reg8::E => ctx.regs.set_e(val),
            Reg8::H => ctx.regs.set_h(val),
            Reg8::L => ctx.regs.set_l(val),
        }
    }
}

pub enum Reg16 { BC, DE, HL, SP }

impl SrcOp<u16> for Reg16 {
    fn get<B: Bus>(&self, ctx: &Context<B>) -> u16 { 
        match self {
            Reg16::BC => ctx.regs.bc(),
            Reg16::DE => ctx.regs.de(),
            Reg16::HL => ctx.regs.hl(),
            Reg16::SP => ctx.regs.sp(),
        }
    }
}

impl DestOp<u16> for Reg16 {
    fn set<B: Bus>(&self, ctx: &mut Context<B>, val: u16) {
        match self {
            Reg16::BC => ctx.regs.set_bc(val),
            Reg16::DE => ctx.regs.set_de(val),
            Reg16::HL => ctx.regs.set_hl(val),
            Reg16::SP => ctx.regs.set_sp(val),
        }
    }
}

pub struct Imm8 { offset: u16 }

impl Imm8 {
    pub fn with_offset(offset: u16) -> Self {
        Self { offset }
    }
}

impl SrcOp<u8> for Imm8 {
    fn get<B: Bus>(&self, ctx: &Context<B>) -> u8 { 
        let addr = ctx.regs.pc() + self.offset;
        ctx.bus.mem_read(addr)
    }
}

pub struct Imm16 { offset: u16 }

impl Imm16 {
    pub fn with_offset(offset: u16) -> Self {
        Self { offset }
    }
}

impl SrcOp<u16> for Imm16 {
    fn get<B: Bus>(&self, ctx: &Context<B>) -> u16 { 
        let addr = ctx.regs.pc() + self.offset;
        ctx.bus.mem_read_word(addr)
    }
}

pub struct Ind8<T: SrcOp<u16>>(pub T);

impl<T: SrcOp<u16>> SrcOp<u8> for Ind8<T> {
    fn get<B: Bus>(&self, ctx: &Context<B>) -> u8 { 
        let addr = self.0.get(ctx);
        ctx.bus.mem_read(addr)
    }
}

impl<T: SrcOp<u16>> DestOp<u8> for Ind8<T> {
    fn set<B: Bus>(&self, ctx: &mut Context<B>, val: u8) { 
        let addr = self.0.get(ctx);
        ctx.bus.mem_write(addr, val)
    }
}

pub struct Ind16<T: SrcOp<u16>>(pub T);

impl<T: SrcOp<u16>> SrcOp<u16> for Ind16<T> {
    fn get<B: Bus>(&self, ctx: &Context<B>) -> u16 { 
        let addr = self.0.get(ctx);
        ctx.bus.mem_read_word(addr)
    }
}

impl<T: SrcOp<u16>> DestOp<u16> for Ind16<T> {
    fn set<B: Bus>(&self, ctx: &mut Context<B>, val: u16) { 
        let addr = self.0.get(ctx);
        ctx.bus.mem_write_word(addr, val)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cpu::z81::bus::FakeBus;

    #[test]
    fn test_reg8() {
        let mut regs= Registers::new();
        let mut bus = FakeBus::new();
        let mut ctx = Context::from(&mut bus, &mut regs);

        ctx.regs.set_a(0x42);
        assert_eq!(Reg8::A.get(&ctx), 0x42);
        Reg8::A.set(&mut ctx, 0x24);
        assert_eq!(ctx.regs.a(), 0x24);
    }    
}