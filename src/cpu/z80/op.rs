use crate::cpu::z80::bus::Bus;
use crate::cpu::z80::reg::Registers;

/// The context in which an operand evaluates in.
pub struct Context<'a, B: Bus> {
    pub bus: &'a mut B,
    pub regs: &'a mut Registers,
}

impl<'a, B: Bus> Context<'a, B> {
    pub fn from(bus: &'a mut B, regs: &'a mut Registers) -> Self {
        Self { bus, regs }
    }
}

/// A source operaand that can get values of type T.
pub trait SrcOp<T> {
    fn get<B: Bus>(&self, ctx: &Context<B>) -> T;
}

/// A destination operand that can get and set values of type T.
pub trait DestOp<T> : SrcOp<T> {
    fn set<B: Bus>(&self, ctx: &mut Context<B>, val: T);
}

/// A literal 8-bit value that can be used as source operand.
pub struct Lit8(pub u8);

impl SrcOp<u8> for Lit8 {
    fn get<B: Bus>(&self, _: &Context<B>) -> u8 { self.0 }
}

/// A 8-bit CPU register that can act as source and destination operand.
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

/// A 16-bit CPU register that can act as source and destination operand.
pub enum Reg16 { AF, BC, DE, HL, SP, AF_ }

impl SrcOp<u16> for Reg16 {
    fn get<B: Bus>(&self, ctx: &Context<B>) -> u16 { 
        match self {
            Reg16::AF => ctx.regs.af(),
            Reg16::BC => ctx.regs.bc(),
            Reg16::DE => ctx.regs.de(),
            Reg16::HL => ctx.regs.hl(),
            Reg16::SP => ctx.regs.sp(),
            Reg16::AF_ => ctx.regs.af_(),
        }
    }
}

impl DestOp<u16> for Reg16 {
    fn set<B: Bus>(&self, ctx: &mut Context<B>, val: u16) {
        match self {
            Reg16::AF => ctx.regs.set_af(val),
            Reg16::BC => ctx.regs.set_bc(val),
            Reg16::DE => ctx.regs.set_de(val),
            Reg16::HL => ctx.regs.set_hl(val),
            Reg16::SP => ctx.regs.set_sp(val),
            Reg16::AF_ => ctx.regs.set_af_(val),
        }
    }
}

/// An 8-bit immediate source operand that comes after the opcode of its instruction.
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

/// A 16-bit immediate source operand that comes after the opcode of its instruction.
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

/// An 8-bit indirect operand that indicates an address in memory where data is located.
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

/// A 16-bit indirect operand that indicates an address in memory where data is located.
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
    use crate::cpu::z80::bus::FakeBus;
    use rstest::*;

    #[fixture]
    fn fixture() -> Fixture {
        Fixture {
            regs: Registers::new(),
            bus: FakeBus::new(),
        }
    }

    struct Fixture {
        regs: Registers,
        bus: FakeBus,
    }

    impl Fixture {
        fn context(&mut self) -> Context<FakeBus> {
            Context::from(&mut self.bus, &mut self.regs)
        }
    }

    #[rstest]
    #[case(Reg8::A, |regs: &Registers| regs.a(), |regs: &mut Registers, val: u8| regs.set_a(val))]
    #[case(Reg8::B, |regs: &Registers| regs.b(), |regs: &mut Registers, val: u8| regs.set_b(val))]
    #[case(Reg8::C, |regs: &Registers| regs.c(), |regs: &mut Registers, val: u8| regs.set_c(val))]
    #[case(Reg8::D, |regs: &Registers| regs.d(), |regs: &mut Registers, val: u8| regs.set_d(val))]
    #[case(Reg8::E, |regs: &Registers| regs.e(), |regs: &mut Registers, val: u8| regs.set_e(val))]
    #[case(Reg8::H, |regs: &Registers| regs.h(), |regs: &mut Registers, val: u8| regs.set_h(val))]
    #[case(Reg8::L, |regs: &Registers| regs.l(), |regs: &mut Registers, val: u8| regs.set_l(val))]
    fn test_reg8(
        mut fixture: Fixture, 
        #[case] reg: Reg8, 
        #[case] get: fn(&Registers) -> u8, 
        #[case] set: fn(&mut Registers, u8),
    ) {
        let mut ctx = fixture.context();
        set(ctx.regs, 0x42);
        assert_eq!(reg.get(&ctx), 0x42);

        reg.set(&mut ctx, 0x24);
        assert_eq!(get(ctx.regs), 0x24);
    }

    #[rstest]
    #[case(Reg16::AF, |regs: &Registers| regs.af(), |regs: &mut Registers, val: u16| regs.set_af(val))]
    #[case(Reg16::BC, |regs: &Registers| regs.bc(), |regs: &mut Registers, val: u16| regs.set_bc(val))]
    #[case(Reg16::DE, |regs: &Registers| regs.de(), |regs: &mut Registers, val: u16| regs.set_de(val))]
    #[case(Reg16::HL, |regs: &Registers| regs.hl(), |regs: &mut Registers, val: u16| regs.set_hl(val))]
    #[case(Reg16::AF_, |regs: &Registers| regs.af_(), |regs: &mut Registers, val: u16| regs.set_af_(val))]
    fn test_reg16(
        mut fixture: Fixture, 
        #[case] reg: Reg16, 
        #[case] get: fn(&Registers) -> u16, 
        #[case] set: fn(&mut Registers, u16),
    ) {
        let mut ctx = fixture.context();
        set(ctx.regs, 0x4224);
        assert_eq!(reg.get(&ctx), 0x4224);

        reg.set(&mut ctx, 0xABCD);
        assert_eq!(get(ctx.regs), 0xABCD);
    }

    #[rstest]
    #[case(1)]
    #[case(2)]
    #[case(3)]
    fn test_imm8(mut fixture: Fixture, #[case] offset: u16) {
        let ctx = fixture.context();
        let op = Imm8::with_offset(offset);

        ctx.regs.set_pc(0x4000);
        ctx.bus.mem_write(0x4000 + offset, 0x42);

        assert_eq!(op.get(&ctx), 0x42);
    }

    #[rstest]
    #[case(1)]
    #[case(2)]
    #[case(3)]
    fn test_imm16(mut fixture: Fixture, #[case] offset: u16) {
        let ctx = fixture.context();
        let op = Imm16::with_offset(offset);

        ctx.regs.set_pc(0x4000);
        ctx.bus.mem_write_word(0x4000 + offset, 0xABCD);

        assert_eq!(op.get(&ctx), 0xABCD);
    }

    #[rstest]
    #[case(Reg16::BC)]
    #[case(Reg16::DE)]
    #[case(Reg16::HL)]
    #[case(Reg16::SP)]
    fn test_ind8(mut fixture: Fixture, #[case] reg: Reg16) {
        let mut ctx = fixture.context();
        
        reg.set(&mut ctx, 0x4000);        
        ctx.bus.mem_write(0x4000, 0x42);

        let op = Ind8(reg);
        assert_eq!(op.get(&ctx), 0x42);
    }

    #[rstest]
    #[case(Reg16::BC)]
    #[case(Reg16::DE)]
    #[case(Reg16::HL)]
    #[case(Reg16::SP)]
    fn test_ind16(mut fixture: Fixture, #[case] reg: Reg16) {
        let mut ctx = fixture.context();
        
        reg.set(&mut ctx, 0x4000);        
        ctx.bus.mem_write_word(0x4000, 0xABCD);

        let op = Ind16(reg);
        assert_eq!(op.get(&ctx), 0xABCD);
    }
}