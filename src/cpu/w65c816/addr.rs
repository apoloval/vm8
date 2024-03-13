use std::fmt::{self, Display, Formatter};

use super::{status::Flag, Addr, AddrWrap, Bus, CPU};

macro_rules! cpu_op16 {
    ($cpu:expr) => {
        if $cpu.regs.status_flag_is_reset(Flag::M) { 1 } else { 0 }
    };
}

macro_rules! cpu_dl0 {
    ($cpu:expr) => {
        if $cpu.regs.dl() != 0 { 1 } else { 0 }
    };
}


#[derive(Clone, Copy)]
pub enum Mode {
    Absolute(u16),                      // a
    AbsoluteIndexedX(u16),              // a,X
    AbsoluteIndexedY(u16),              // a,Y
    AbsoluteLong(u8, u16),              // al       --> 65C816 only
    AbsoluteLongIndexed(u8, u16),       // al,X     --> 65C816 only
    Direct(u8),                         // d     
    DirectIndexedIndirect(u8),          // (d,X)
    DirectIndexedX(u8),                 // d,X
    DirectIndexedY(u8),                 // d,Y
    DirectIndirect(u8),                 // (d)      --> 65C816 only
    DirectIndirectIndexed(u8),          // (d),Y
    DirectIndirectLongIndexed(u8),      // [d],Y    --> 65C816 only
    DirectIndirectLong(u8),             // [d]      --> 65C816 only
    Immediate(u16),                     // #i
    StackRelative(u8),                  // d,S      --> 65C816 only
    StackRelativeIndirectIndexed(u8),   // (d,S),Y  --> 65C816 only
}

pub struct ModeEval {
    pub val: u16,
    pub cycles: u64,
    pub bytes: u16,
}

impl Mode {
    pub fn eval<B: Bus>(self, cpu: &mut CPU, bus: &B) -> ModeEval {
        match self {
            Mode::Absolute(addr) =>
                ModeEval {
                    val: bus.read_word(
                        Addr::from(cpu.regs.dbr(), addr),
                        AddrWrap::Long,
                    ),
                    cycles: 4 + cpu_op16!(cpu),
                    bytes: 3,
                },
            Mode::AbsoluteIndexedX(addr) => {
                let ptr1 = Addr::from(cpu.regs.dbr(), addr);
                let ptr2 = ptr1.wrapping_add(cpu.regs.x(), AddrWrap::Long);
                ModeEval {
                    val: bus.read_word(ptr2, AddrWrap::Long),
                    cycles: 4 + cpu_op16!(cpu) + Self::addr_page_crossed(cpu, ptr1, ptr2),
                    bytes: 3,
                }
            },
            Mode::AbsoluteIndexedY(addr) => {
                let ptr1 = Addr::from(cpu.regs.dbr(), addr);
                let ptr2 = ptr1.wrapping_add(cpu.regs.y(), AddrWrap::Long);
                ModeEval {
                    val: bus.read_word(ptr2, AddrWrap::Long),
                    cycles: 4 + cpu_op16!(cpu) + Self::addr_page_crossed(cpu, ptr1, ptr2),
                    bytes: 3,
                }
            },
            Mode::AbsoluteLong(bank, offset) =>
                ModeEval {
                    val: bus.read_word(Addr::from(bank, offset), AddrWrap::Long),
                    cycles: 5 + cpu_op16!(cpu),
                    bytes: 4,
                },
            Mode::AbsoluteLongIndexed(bank, offset) => {
                let addr = Addr::from(bank, offset)
                    .wrapping_add(cpu.regs.x(), AddrWrap::Long);
                ModeEval {
                    val: bus.read_word(addr, AddrWrap::Long),
                    cycles: 5 + cpu_op16!(cpu),
                    bytes: 4,
                }
            },
            Mode::Direct(dir) =>
                ModeEval {
                    val: cpu.direct_word(bus, dir, 0),
                    cycles: 3 + cpu_op16!(cpu) + cpu_dl0!(cpu),
                    bytes: 2,
                },
            Mode::DirectIndexedX(dir) =>
                ModeEval {
                    val: cpu.direct_word(bus, dir, cpu.regs.x()),
                    cycles: 4 + cpu_op16!(cpu) + cpu_dl0!(cpu),
                    bytes: 2,
                },
            Mode::DirectIndexedY(dir) =>
                ModeEval {
                    val: cpu.direct_word(bus, dir, cpu.regs.y()),
                    cycles: 4 + cpu_op16!(cpu) + cpu_dl0!(cpu),
                    bytes: 2,
                },
            Mode::DirectIndexedIndirect(dir) =>
                ModeEval {
                    val: bus.read_word(
                        cpu.direct_ptr(bus, dir, cpu.regs.x()), 
                        AddrWrap::Long,
                    ),
                    cycles: 6 + cpu_op16!(cpu) + cpu_dl0!(cpu),
                    bytes: 2,
                },
            Mode::DirectIndirect(dir) => {
                println!("DirectIndirect: {:?}", cpu.direct_ptr(bus, dir, 0));
                ModeEval {
                    val: bus.read_word(
                        cpu.direct_ptr(bus, dir, 0),
                        AddrWrap::Long,
                    ),
                    cycles: 5 + cpu_op16!(cpu) + cpu_dl0!(cpu),
                    bytes: 2,
                }
            },
            Mode::DirectIndirectIndexed(dir) => {
                let ptr1 = cpu.direct_ptr(bus, dir, 0);
                let ptr2 = ptr1.wrapping_add(cpu.regs.y(), AddrWrap::Long);

                ModeEval {
                    val: bus.read_word(ptr2, AddrWrap::Long),
                    cycles: 5 + 
                        cpu_op16!(cpu) + 
                        cpu_dl0!(cpu) + 
                        Self::addr_page_crossed(cpu, ptr1, ptr2),
                    bytes: 2,
                }
            },
            Mode::DirectIndirectLong(offset) => {
                let addr= cpu.direct_ptr_long(bus, offset, 0);
                ModeEval {
                    val: bus.read_word(addr, AddrWrap::Long),
                    cycles: 6 + cpu_op16!(cpu) + cpu_dl0!(cpu),
                    bytes: 2,
                }
            },
            Mode::DirectIndirectLongIndexed(offset) => {
                let addr = cpu.direct_ptr_long(bus, offset, 0)
                    .wrapping_add(cpu.regs.y(), AddrWrap::Word);
                ModeEval {
                    val: bus.read_word(addr, AddrWrap::Long,),
                    cycles: 6 + cpu_op16!(cpu) + cpu_dl0!(cpu),
                    bytes: 2,
                }
            },
            Mode::Immediate(value) => 
                ModeEval {
                    val: value,
                    cycles: 2 + cpu_op16!(cpu),
                    bytes: 2 + cpu_op16!(cpu),
                },
            Mode::StackRelative(offset) => {
                ModeEval {
                    val: cpu.stack_word(bus, offset as u16),
                    cycles: 4 + cpu_op16!(cpu),
                    bytes: 2,
                }
            },
            Mode::StackRelativeIndirectIndexed(offset) => {
                let addr = Addr::from(
                    cpu.regs.dbr(),
                    cpu.stack_word(bus, offset as u16),
                );
                let ptr = addr.wrapping_add(cpu.regs.y(), AddrWrap::Word);
                ModeEval {
                    val: bus.read_word(ptr, AddrWrap::Long,),
                    cycles: 7 + cpu_op16!(cpu),
                    bytes: 2,
                }
            },
        }
    }

    fn addr_page_crossed(cpu: &CPU, ptr1: Addr, ptr2: Addr) -> u64 {
        if !ptr1.same_page(ptr2) || cpu.regs.status_flag_is_reset(Flag::X) {
            1
        } else {
            0
        }
    }
}

impl Display for Mode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Mode::Absolute(addr) => write!(f, "${:04X}", addr),
            Mode::AbsoluteIndexedX(addr) => write!(f, "${:04X},X", addr),
            Mode::AbsoluteIndexedY(addr) => write!(f, "${:04X},Y", addr),
            Mode::AbsoluteLong(bank, addr) => write!(f, "${:02X}{:04X}", bank, addr),
            Mode::AbsoluteLongIndexed(bank, addr) => write!(f, "${:02X}{:04X},X", bank, addr),
            Mode::Direct(dir) => write!(f, "${:02X}", dir),
            Mode::DirectIndirect(dir) => write!(f, "(${:02X})", dir),
            Mode::DirectIndexedIndirect(offset) => write!(f, "(${:02X},X)", offset),
            Mode::DirectIndexedX(offset) => write!(f, "${:02X},X", offset),
            Mode::DirectIndexedY(offset) => write!(f, "${:02X},Y", offset),
            Mode::DirectIndirectIndexed(dir) => write!(f, "(${:02X}),Y", dir),
            Mode::DirectIndirectLong(offset) => write!(f, "[${:02X}]", offset),
            Mode::DirectIndirectLongIndexed(dir) => write!(f, "[${:02X}],Y", dir),
            Mode::Immediate(value) => write!(f, "#${:04X}", value),
            Mode::StackRelative(offset) => write!(f, "${:02X},S", offset),
            Mode::StackRelativeIndirectIndexed(offset) => write!(f, "(${:02X},S),Y", offset),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::w65c816::bus;

    use rstest::*;

    #[rstest]
    /****************/
    /* Absolute : a */
    /****************/
    #[case::absolute_emulated(
        "P.E:1",                                            // cpu
        "001234:AB",                                        // bus
        Mode::Absolute(0x1234),                             // addr
        ModeEval{val: 0x00AB, cycles: 4, bytes: 3},         // expected
    )]
    #[case::absolute_native_8bit(
        "P.E:0,P.M:1,DBR:12",                               // cpu
        "123456:AB",                                        // bus
        Mode::Absolute(0x3456),                             // addr
        ModeEval{val: 0xAB, cycles: 4, bytes: 3},           // expected
    )]
    #[case::absolute_native_16bit(
        "P.E:0,DBR:12",                                     // cpu
        "123456:CDAB",                                      // bus
        Mode::Absolute(0x3456),                             // addr
        ModeEval{val: 0xABCD, cycles: 5, bytes: 3},         // expected
    )]

    /****************************/
    /* Absolute indexed X : a,X */
    /****************************/
    #[case::absolute_indexed_x_emulated(
        "P.E:1,X:12",                                       // cpu
        "001246:AB",                                        // bus
        Mode::AbsoluteIndexedX(0x1234),                     // addr
        ModeEval{val: 0x00AB, cycles: 4, bytes: 3},         // expected
    )]
    #[case::absolute_indexed_x_native_8bit(
        "P.E:0,P.M:1,P.X:1,DBR:12,X:12",                    // cpu
        "123468:AB",                                        // bus
        Mode::AbsoluteIndexedX(0x3456),                     // addr
        ModeEval{val: 0xAB, cycles: 4, bytes: 3},           // expected
    )]
    #[case::absolute_indexed_x_native_8bit_16bitidx(
        "P.E:0,P.M:1,P.X:0,DBR:12,X:12",                    // cpu
        "123468:AB",                                        // bus
        Mode::AbsoluteIndexedX(0x3456),                     // addr
        ModeEval{val: 0xAB, cycles: 5, bytes: 3},           // expected
    )]
    #[case::absolute_indexed_x_native_16bit(
        "P.E:0,DBR:12,X:12",                                // cpu
        "123468:CDAB",                                      // bus
        Mode::AbsoluteIndexedX(0x3456),                     // addr
        ModeEval{val: 0xABCD, cycles: 6, bytes: 3},         // expected
    )]
    #[case::absolute_indexed_x_native_16bit_8bitidx(
        "P.E:0,P.X:1,DBR:12,X:12",                          // cpu
        "123468:CDAB",                                      // bus
        Mode::AbsoluteIndexedX(0x3456),                     // addr
        ModeEval{val: 0xABCD, cycles: 5, bytes: 3},         // expected
    )]

    /**********************/
    /* Absolute long : al */
    /**********************/
    #[case::absolute_long_8bit(
        "P.E:0,P.M:1",                                      // cpu
        "123456:AB",                                        // bus
        Mode::AbsoluteLong(0x12, 0x3456),                   // addr
        ModeEval{val: 0x00AB, cycles: 5, bytes: 4},         // expected
    )]
    #[case::absolute_long_16bit(
        "P.E:0",                                            // cpu
        "123456:CDAB",                                      // bus
        Mode::AbsoluteLong(0x12, 0x3456),                   // addr
        ModeEval{val: 0xABCD, cycles: 6, bytes: 4},         // expected
    )]

    /********************************/
    /* Absolute long indexed : al,X */
    /********************************/
    #[case::absolute_long_indexed_8bit(
        "P.E:0,P.M:1,X:12",                                 // cpu
        "123468:AB",                                        // bus
        Mode::AbsoluteLongIndexed(0x12, 0x3456),            // addr
        ModeEval{val: 0x00AB, cycles: 5, bytes: 4},         // expected
    )]
    #[case::absolute_long_indexed_16bit(
        "P.E:0,X:12",                                       // cpu
        "123468:CDAB",                                      // bus
        Mode::AbsoluteLongIndexed(0x12, 0x3456),            // addr
        ModeEval{val: 0xABCD, cycles: 6, bytes: 4},         // expected
    )]

    /**************/
    /* Direct : d */
    /**************/
    #[case::direct_emulated(
        "P.E:1,DP:FF10",                                    // cpu
        "00FF30:12",                                        // bus
        Mode::Direct(0x20),                                 // addr
        ModeEval{val: 0x0012, cycles: 4, bytes: 2},         // expected
    )]
    #[case::direct_emulated_dl0(
        "P.E:1,DP:FF00",                                    // cpu
        "00FF20:12",                                        // bus
        Mode::Direct(0x20),                                 // addr
        ModeEval{val: 0x0012, cycles: 3, bytes: 2},         // expected
    )]
    #[case::direct_emulated_page_wrapping(
        "P.E:1,DP:FFF0",                                    // cpu
        "000010:12",                                        // bus
        Mode::Direct(0x20),                                 // addr
        ModeEval{val: 0x0012, cycles: 4, bytes: 2},         // expected
    )]
    #[case::direct_native_8bit(
        "P.E:0,P.M:1,DP:FF10",                              // cpu
        "00FF30:12",                                        // bus
        Mode::Direct(0x20),                                 // addr
        ModeEval{val: 0x0012, cycles: 4, bytes: 2},         // expected
    )]
    #[case::direct_native_8bit_dl0(
        "P.E:0,P.M:1,DP:FF00",                              // cpu
        "00FF20:12",                                        // bus
        Mode::Direct(0x20),                                 // addr
        ModeEval{val: 0x0012, cycles: 3, bytes: 2},         // expected
    )]
    #[case::direct_native_8bit_bank_wrapping(
        "P.E:0,P.M:1,DP:FFF0",                              // cpu
        "000010:12",                                        // bus
        Mode::Direct(0x20),                                 // addr
        ModeEval{val: 0x0012, cycles: 4, bytes: 2},         // expected
    )]
    #[case::direct_native_16bit(
        "P.E:0,P.M:0,DP:FF10",                              // cpu
        "00FF30:3412",                                      // bus
        Mode::Direct(0x20),                                 // addr
        ModeEval{val: 0x1234, cycles: 5, bytes: 2},         // expected
    )]
    #[case::direct_native_16bit_dl0(
        "P.E:0,P.M:0,DP:FF00",                              // cpu
        "00FF20:3412",                                      // bus
        Mode::Direct(0x20),                                 // addr
        ModeEval{val: 0x1234, cycles: 4, bytes: 2},         // expected
    )]
    #[case::direct_native_16bit_wrapping(
        "P.E:0,P.M:0,DP:FFF0",                              // cpu
        "000010:3412",                                      // bus
        Mode::Direct(0x20),                                 // addr
        ModeEval{val: 0x1234, cycles: 5, bytes: 2},         // expected
    )]

    /**************************/
    /* Direct indexed X : d,X */
    /**************************/
    #[case::direct_indexed_x_emulated(
        "P.E:1,DP:FF10,X:12",                               // cpu
        "00FF42:12",                                        // bus
        Mode::DirectIndexedX(0x20),                         // addr
        ModeEval{val: 0x0012, cycles: 5, bytes: 2},         // expected
    )]
    #[case::direct_indexed_x_emulated_dl0(
        "P.E:1,DP:FF00,X:12",                               // cpu
        "00FF32:12",                                        // bus
        Mode::DirectIndexedX(0x20),                         // addr
        ModeEval{val: 0x0012, cycles: 4, bytes: 2},         // expected
    )]
    #[case::direct_indexed_x_emulated_page_wrapping(
        "P.E:1,DP:FFF0,X:12",                               // cpu
        "000022:12",                                        // bus
        Mode::DirectIndexedX(0x20),                         // addr
        ModeEval{val: 0x0012, cycles: 5, bytes: 2},         // expected
    )]
    #[case::direct_indexed_x_native_8bit(
        "P.E:0,P.M:1,DP:FF10,X:12",                         // cpu
        "00FF42:12",                                        // bus
        Mode::DirectIndexedX(0x20),                         // addr
        ModeEval{val: 0x0012, cycles: 5, bytes: 2},         // expected
    )]
    #[case::direct_indexed_x_native_8bit_dl0(
        "P.E:0,P.M:1,DP:FF00,X:12",                         // cpu
        "00FF32:12",                                        // bus
        Mode::DirectIndexedX(0x20),                         // addr
        ModeEval{val: 0x0012, cycles: 4, bytes: 2},         // expected
    )]
    #[case::direct_indexed_x_native_8bit_bank_wrapping(
        "P.E:0,P.M:1,DP:FFF0,X:12",                         // cpu
        "000022:12",                                        // bus
        Mode::DirectIndexedX(0x20),                         // addr
        ModeEval{val: 0x0012, cycles: 5, bytes: 2},         // expected
    )]
    #[case::direct_indexed_x_native_16bit(
        "P.E:0,P.M:0,DP:FF10,X:12",                         // cpu
        "00FF42:3412",                                      // bus
        Mode::DirectIndexedX(0x20),                         // addr
        ModeEval{val: 0x1234, cycles: 6, bytes: 2},         // expected
    )]
    #[case::direct_indexed_x_native_16bit_dl0(
        "P.E:0,P.M:0,DP:FF00,X:12",                         // cpu
        "00FF32:3412",                                      // bus
        Mode::DirectIndexedX(0x20),                         // addr
        ModeEval{val: 0x1234, cycles: 5, bytes: 2},         // expected
    )]
    #[case::direct_indexed_x_native_16bit_wrapping(
        "P.E:0,P.M:0,DP:FFF0,X:12",                         // cpu
        "000022:3412",                                      // bus
        Mode::DirectIndexedX(0x20),                         // addr
        ModeEval{val: 0x1234, cycles: 6, bytes: 2},         // expected
    )]

    /**************************/
    /* Direct indexed Y : d,Y */
    /**************************/
    #[case::direct_indexed_y_emulated(
        "P.E:1,DP:FF10,Y:12",                               // cpu
        "00FF42:12",                                        // bus
        Mode::DirectIndexedY(0x20),                         // addr
        ModeEval{val: 0x0012, cycles: 5, bytes: 2},         // expected
    )]
    #[case::direct_indexed_y_emulated_dl0(
        "P.E:1,DP:FF00,Y:12",                               // cpu
        "00FF32:12",                                        // bus
        Mode::DirectIndexedY(0x20),                         // addr
        ModeEval{val: 0x0012, cycles: 4, bytes: 2},         // expected
    )]
    #[case::direct_indexed_y_emulated_page_wrapping(
        "P.E:1,DP:FFF0,Y:12",                               // cpu
        "000022:12",                                        // bus
        Mode::DirectIndexedY(0x20),                         // addr
        ModeEval{val: 0x0012, cycles: 5, bytes: 2},         // expected
    )]
    #[case::direct_indexed_y_native_8bit(
        "P.E:0,P.M:1,DP:FF10,Y:12",                         // cpu
        "00FF42:12",                                        // bus
        Mode::DirectIndexedY(0x20),                         // addr
        ModeEval{val: 0x0012, cycles: 5, bytes: 2},         // expected
    )]
    #[case::direct_indexed_y_native_8bit_dl0(
        "P.E:0,P.M:1,DP:FF00,Y:12",                         // cpu
        "00FF32:12",                                        // bus
        Mode::DirectIndexedY(0x20),                         // addr
        ModeEval{val: 0x0012, cycles: 4, bytes: 2},         // expected
    )]
    #[case::direct_indexed_y_native_8bit_bank_wrapping(
        "P.E:0,P.M:1,DP:FFF0,Y:12",                         // cpu
        "000022:12",                                        // bus
        Mode::DirectIndexedY(0x20),                         // addr
        ModeEval{val: 0x0012, cycles: 5, bytes: 2},         // expected
    )]
    #[case::direct_indexed_y_native_16bit(
        "P.E:0,P.M:0,DP:FF10,Y:12",                         // cpu
        "00FF42:3412",                                      // bus
        Mode::DirectIndexedY(0x20),                         // addr
        ModeEval{val: 0x1234, cycles: 6, bytes: 2},         // expected
    )]
    #[case::direct_indexed_y_native_16bit_dl0(
        "P.E:0,P.M:0,DP:FF00,Y:12",                         // cpu
        "00FF32:3412",                                      // bus
        Mode::DirectIndexedY(0x20),                         // addr
        ModeEval{val: 0x1234, cycles: 5, bytes: 2},         // expected
    )]
    #[case::direct_indexed_y_native_16bit_wrapping(
        "P.E:0,P.M:0,DP:FFF0,Y:12",                         // cpu
        "000022:3412",                                      // bus
        Mode::DirectIndexedY(0x20),                         // addr
        ModeEval{val: 0x1234, cycles: 6, bytes: 2},         // expected
    )]

    /***********************************/
    /* Direct indexed indirect : (d,X) */
    /***********************************/
    #[case::direct_indexed_indirect_emulated(
        "P.E:1,DP:1040,X:12",                               // cpu
        "001056:8040,004080:3412",                          // bus
        Mode::DirectIndexedIndirect(0x04),                  // addr
        ModeEval{val: 0x1234, cycles: 7, bytes: 2},         // expected
    )]
    #[case::direct_indexed_indirect_emulated_dl0(
        "P.E:1,DP:1000,X:12",                               // cpu
        "001016:8040,004080:3412",                          // bus
        Mode::DirectIndexedIndirect(0x04),                  // addr
        ModeEval{val: 0x1234, cycles: 6, bytes: 2},         // expected
    )]
    #[case::direct_indexed_indirect_emulated_page_wrapping(
        "P.E:1,DP:1000,X:FF",                               // cpu
        "001003:8040,004080:3412",                          // bus
        Mode::DirectIndexedIndirect(0x04),                  // addr
        ModeEval{val: 0x1234, cycles: 6, bytes: 2},         // expected
    )]
    #[case::direct_indexed_indirect_emulated_bank_wrapping(
        "P.E:1,DP:FFF0,X:20",                               // cpu
        "000014:8040,004080:3412",                          // bus
        Mode::DirectIndexedIndirect(0x04),                  // addr
        ModeEval{val: 0x1234, cycles: 7, bytes: 2},         // expected
    )]
    #[case::direct_indexed_indirect_native_16bit(
        "P.E:0,P.M:0,DP:1040,X:12",                         // cpu
        "001056:8040,004080:3412",                          // bus
        Mode::DirectIndexedIndirect(0x04),                  // addr
        ModeEval{val: 0x1234, cycles: 8, bytes: 2},         // expected
    )]
    #[case::direct_indexed_indirect_native_16bit_bank_wrapping(
        "P.E:0,DP:FFF0,X:20",                               // cpu
        "000014:8040,004080:3412",                          // bus
        Mode::DirectIndexedIndirect(0x04),                  // addr
        ModeEval{val: 0x1234, cycles: 8, bytes: 2},         // expected
    )]

    /*************************/
    /* Direct indirect : (d) */
    /*************************/
    #[case::direct_indirect_emulated(
        "P.E:1,DP:1040",                                    // cpu
        "001044:8040,004080:3412",                          // bus
        Mode::DirectIndirect(0x04),                         // addr
        ModeEval{val: 0x1234, cycles: 6, bytes: 2},         // expected
    )]
    #[case::direct_indirect_emulated_dl0(
        "P.E:1,DP:1000",                                    // cpu
        "001004:8040,004080:3412",                          // bus
        Mode::DirectIndirect(0x04),                         // addr
        ModeEval{val: 0x1234, cycles: 5, bytes: 2},         // expected
    )]
    #[case::direct_indirect_emulated_page_wrapping(
        "P.E:1,DP:1000",                                    // cpu
        "001000:40,0010FF:80,004080:3412",                  // bus
        Mode::DirectIndirect(0xFF),                         // addr
        ModeEval{val: 0x1234, cycles: 5, bytes: 2},         // expected
    )]
    #[case::direct_indirect_emulated_bank_wrapping(
        "P.E:1,DP:FFF0",                                    // cpu
        "000014:8040,004080:3412",                          // bus
        Mode::DirectIndirect(0x24),                         // addr
        ModeEval{val: 0x1234, cycles: 6, bytes: 2},         // expected
    )]
    #[case::direct_indirect_native_8bit(
        "P.E:0,P.M:1,DP:1040",                              // cpu
        "001044:8040,004080:3412",                          // bus
        Mode::DirectIndirect(0x04),                         // addr
        ModeEval{val: 0x1234, cycles: 6, bytes: 2},         // expected
    )]
    #[case::direct_indirect_native_8bit_dl0(
        "P.E:0,P.M:1,DP:1000",                              // cpu
        "001004:8040,004080:3412",                          // bus
        Mode::DirectIndirect(0x04),                         // addr
        ModeEval{val: 0x1234, cycles: 5, bytes: 2},         // expected
    )]
    #[case::direct_indirect_native_16bit(
        "P.E:0,P.M:0,DP:1040",                              // cpu
        "001044:8040,004080:3412",                          // bus
        Mode::DirectIndirect(0x04),                         // addr
        ModeEval{val: 0x1234, cycles: 7, bytes: 2},         // expected
    )]
    #[case::direct_indirect_native_16bit_dl0(
        "P.E:0,P.M:0,DP:1000",                              // cpu
        "001004:8040,004080:3412",                          // bus
        Mode::DirectIndirect(0x04),                         // addr
        ModeEval{val: 0x1234, cycles: 6, bytes: 2},         // expected
    )]
    #[case::direct_indirect_native_16bit_bank_wrapping(
        "P.E:0,DP:FFF0",                                    // cpu
        "000014:8040,004080:3412",                          // bus
        Mode::DirectIndirect(0x24),                         // addr
        ModeEval{val: 0x1234, cycles: 7, bytes: 2},         // expected
    )]

    /***********************************/
    /* Direct indirect indexed : (d),Y */
    /***********************************/
    #[case::direct_indirect_indexed_emulated(
        "P.E:1,DP:1040,Y:12",                               // cpu
        "001044:8040,004092:3412",                          // bus
        Mode::DirectIndirectIndexed(0x04),                  // addr
        ModeEval{val: 0x1234, cycles: 6, bytes: 2},         // expected
    )]
    #[case::direct_indirect_indexed_emulated_dl0(
        "P.E:1,DP:1000,Y:12",                               // cpu
        "001004:8040,004092:3412",                          // bus
        Mode::DirectIndirectIndexed(0x04),                  // addr
        ModeEval{val: 0x1234, cycles: 5, bytes: 2},         // expected
    )]
    #[case::direct_indirect_indexed_emulated_page_wrapping(
        "P.E:1,DP:1000,Y:12",                               // cpu
        "001000:40,0010FF:80,004092:3412",                          // bus
        Mode::DirectIndirectIndexed(0xFF),                  // addr
        ModeEval{val: 0x1234, cycles: 5, bytes: 2},         // expected
    )]
    #[case::direct_indirect_indexed_emulated_bank_wrapping(
        "P.E:1,DP:FFF0,Y:12",                               // cpu
        "000014:8040,004092:3412",                          // bus
        Mode::DirectIndirectIndexed(0x24),                  // addr
        ModeEval{val: 0x1234, cycles: 6, bytes: 2},         // expected
    )]
    #[case::direct_indirect_indexed_native_16bit(
        "P.E:0,P.M:0,P.X:0,DP:1040,Y:12",                   // cpu
        "001044:8040,004092:3412",                          // bus
        Mode::DirectIndirectIndexed(0x04),                  // addr
        ModeEval{val: 0x1234, cycles: 8, bytes: 2},         // expected
    )]
    #[case::direct_indirect_indexed_native_16bit_8bitidx(
        "P.E:0,P.M:0,P.X:1,DP:1040,Y:12",                   // cpu
        "001044:8040,004092:3412",                          // bus
        Mode::DirectIndirectIndexed(0x04),                  // addr
        ModeEval{val: 0x1234, cycles: 7, bytes: 2},         // expected
    )]
    #[case::direct_indirect_indexed_native_16bit_8bitidx_wrapping(
        "P.E:0,P.M:0,P.X:1,DP:1040,Y:22",                   // cpu
        "001044:F040,004112:3412",                          // bus
        Mode::DirectIndirectIndexed(0x04),                  // addr
        ModeEval{val: 0x1234, cycles: 8, bytes: 2},         // expected
    )]
    #[case::direct_indirect_indexed_native_16bit_bank_wrapping(
        "P.E:0,DP:FFF0,Y:12",                               // cpu
        "000014:8040,004092:3412",                          // bus
        Mode::DirectIndirectIndexed(0x24),                  // addr
        ModeEval{val: 0x1234, cycles: 8, bytes: 2},         // expected
    )]

    /******************************/
    /* Direct indirect long : [d] */
    /******************************/
    #[case::direct_indirect_long_8bit(
        "P.E:0,P.M:1,DP:FF10",                              // cpu
        "00FF30:563412,123456:AB",                          // bus
        Mode::DirectIndirectLong(0x20),                     // addr
        ModeEval{val: 0xAB, cycles: 7, bytes: 2},           // expected
    )]
    #[case::direct_indirect_long_8bit_dl0(
        "P.E:0,P.M:1,DP:FF00",                              // cpu
        "00FF20:563412,123456:AB",                          // bus
        Mode::DirectIndirectLong(0x20),                     // addr
        ModeEval{val: 0xAB, cycles: 6, bytes: 2},           // expected
    )]
    #[case::direct_indirect_long_16bit(
        "P.E:0,P.M:0,DP:FF10",                              // cpu
        "00FF30:563412,123456:CDAB",                        // bus
        Mode::DirectIndirectLong(0x20),                     // addr
        ModeEval{val: 0xABCD, cycles: 8, bytes: 2},         // expected
    )]
    #[case::direct_indirect_long_16bit_dl0(
        "P.E:0,P.M:0,DP:FF00",                              // cpu
        "00FF20:563412,123456:CDAB",                        // bus
        Mode::DirectIndirectLong(0x20),                     // addr
        ModeEval{val: 0xABCD, cycles: 7, bytes: 2},         // expected
    )]
    #[case::direct_indirect_long_bank_wrapping(
        "P.E:0,P.M:0,DP:FFF0",                              // cpu
        "000010:FFFF12,12FFFF:CDAB",                        // bus
        Mode::DirectIndirectLong(0x20),                     // addr
        ModeEval{val: 0xABCD, cycles: 8, bytes: 2},         // expected
    )]

    /****************************************/
    /* Direct indirect long indexed : [d],X */
    /****************************************/
    #[case::direct_indirect_long_indexed_8bit(
        "P.E:0,P.M:1,DP:FF10,Y:12",                         // cpu
        "00FF30:563412,123468:AB",                          // bus
        Mode::DirectIndirectLongIndexed(0x20),              // addr
        ModeEval{val: 0xAB, cycles: 7, bytes: 2},           // expected
    )]
    #[case::direct_indirect_long_indexed_8bit_dl0(
        "P.E:0,P.M:1,DP:FF00,Y:12",                         // cpu
        "00FF20:563412,123468:AB",                          // bus
        Mode::DirectIndirectLongIndexed(0x20),              // addr
        ModeEval{val: 0xAB, cycles: 6, bytes: 2},           // expected
    )]
    #[case::direct_indirect_long_indexed_16bit(
        "P.E:0,P.M:0,DP:FF10,Y:12",                         // cpu
        "00FF30:563412,123468:CDAB",                        // bus
        Mode::DirectIndirectLongIndexed(0x20),              // addr
        ModeEval{val: 0xABCD, cycles: 8, bytes: 2},         // expected
    )]
    #[case::direct_indirect_long_indexed_16bit_dl0(
        "P.E:0,P.M:0,DP:FF00,Y:12",                         // cpu
        "00FF20:563412,123468:CDAB",                        // bus
        Mode::DirectIndirectLongIndexed(0x20),              // addr
        ModeEval{val: 0xABCD, cycles: 7, bytes: 2},         // expected
    )]
    #[case::direct_indirect_long_indexed_bank_wrapping(
        "P.E:0,P.M:0,DP:FFF0,Y:0F",                         // cpu
        "000010:F0FF12,12FFFF:CDAB",                        // bus
        Mode::DirectIndirectLongIndexed(0x20),              // addr
        ModeEval{val: 0xABCD, cycles: 8, bytes: 2},         // expected
    )]

    /*******************/
    /* Inmmediate : #i */
    /*******************/
    #[case::immediate_8bit(
        "P.E:1",                                            // cpu
        "",                                                 // bus
        Mode::Immediate(0x00AB),                            // addr
        ModeEval{val: 0x00AB, cycles: 2, bytes: 2},         // expected
    )]
    #[case::immediate_16bit(
        "P.E:0",                                            // cpu
        "",                                                 // bus
        Mode::Immediate(0x00AB),                            // addr
        ModeEval{val: 0x00AB, cycles: 3, bytes: 3},         // expected
    )]

    /**************************/
    /* Stack relative : (d,S) */
    /**************************/
    #[case::stack_relative_emulated(
        "P.E:1,SP:FFF0",                                    // cpu
        "0001F2:12",                                        // bus
        Mode::StackRelative(0x02),                          // addr
        ModeEval{val: 0x12, cycles: 4, bytes: 2},           // expected
    )]
    #[case::stack_relative_native_8bit(
        "P.E:0,P.M:1,SP:FFF0",                              // cpu
        "00FFF2:12",                                        // bus
        Mode::StackRelative(0x02),                          // addr
        ModeEval{val: 0x12, cycles: 4, bytes: 2},           // expected
    )]
    #[case::stack_relative_native_16bit(
        "P.E:0,SP:FFF0",                                    // cpu
        "00FFF2:3412",                                      // bus
        Mode::StackRelative(0x02),                          // addr
        ModeEval{val: 0x1234, cycles: 5, bytes: 2},         // expected
    )]
    #[case::stack_relative_bank_wrapping(
        "P.E:0,SP:FFF0",                                    // cpu
        "000002:3412",                                      // bus
        Mode::StackRelative(0x12),                          // addr
        ModeEval{val: 0x1234, cycles: 5, bytes: 2},         // expected
    )]

    /*********************************************/
    /* Stack relative indirect indexed : (d,S),Y */
    /*********************************************/
    #[case::stack_relative_indirect_indexed_emulated(
        "P.E:1,SP:FFF0,Y:08",                               // cpu
        "0001F2:8040,004088:12",                            // bus
        Mode::StackRelativeIndirectIndexed(0x02),           // addr
        ModeEval{val: 0x12, cycles: 7, bytes: 2},           // expected
    )]
    #[case::stack_relative_indirect_indexed_native_8bit(
        "P.E:0,P.M:1,SP:FFF0,Y:08",                         // cpu
        "00FFF2:8040,004088:12",                            // bus
        Mode::StackRelativeIndirectIndexed(0x02),           // addr
        ModeEval{val: 0x12, cycles: 7, bytes: 2},           // expected
    )]
    #[case::stack_relative_indirect_indexed_native_16bit(
        "P.E:0,SP:FFF0,Y:08",                               // cpu
        "00FFF2:8040,004088:3412",                          // bus
        Mode::StackRelativeIndirectIndexed(0x02),           // addr
        ModeEval{val: 0x1234, cycles: 8, bytes: 2},         // expected
    )]
    #[case::stack_relative_indirect_indexed_bank_wrapping(
        "P.E:0,SP:FFF0,Y:08",                               // cpu
        "000002:8040,004088:3412",                          // bus
        Mode::StackRelativeIndirectIndexed(0x12),           // addr
        ModeEval{val: 0x1234, cycles: 8, bytes: 2},         // expected
    )]
    fn test_eval(
        #[case] mut cpu: CPU,
        #[case] bus: bus::Fake,
        #[case] addr: Mode,
        #[case] expected: ModeEval,
    ) {
        let eval = addr.eval(&mut cpu, &bus);
        assert_eq!(eval.val, expected.val);
        assert_eq!(eval.cycles, expected.cycles);
        assert_eq!(eval.bytes, expected.bytes);
    }
}
