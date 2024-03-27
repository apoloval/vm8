use std::fmt::{self, Display, Formatter};

use super::{status::Flag, Addr, AddrWrap, Bus, CPU};

macro_rules! cpu_op16 {
    ($cpu:expr) => {
        if $cpu.regs.status_flag_is_clear(Flag::M) { 1 } else { 0 }
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
    AbsoluteJump(u16),                  // a
    AbsoluteIndirectJump(u16),          // (a)
    AbsoluteIndirectLongJump(u16),      // [a]      --> 65C816 only
    AbsoluteIndexedIndirectJump(u16),   // (a,X)    --> 65C816 only
    AbsoluteLongJump(u8, u16),          // al       --> 65C816 only
    AbsoluteIndexedX(u16),              // a,X
    AbsoluteIndexedY(u16),              // a,Y
    AbsoluteLong(u8, u16),              // al       --> 65C816 only
    AbsoluteLongIndexed(u8, u16),       // al,X     --> 65C816 only
    Accumulator,                        //
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

pub struct ModeRead {
    pub val: u16,
    pub cycles: u64,
    pub prog_bytes: u16,
}

pub struct ModeWrite {
    pub cycles: u64,
}

pub struct ModeJump {
    pub bank: u8,
    pub addr: u16,
    pub jmp_cycles: u64,
    pub jsr_cycles: u64,
    pub prog_bytes: u16,

}

impl Mode {
    pub fn read(self, cpu: &CPU, bus: &impl Bus) -> ModeRead {
        match self {
            Mode::Absolute(addr) =>
                ModeRead {
                    val: bus.read_word(
                        Addr::from(cpu.regs.dbr(), addr),
                        AddrWrap::Long,
                    ),
                    cycles: 4 + cpu_op16!(cpu),
                    prog_bytes: 3,
                },
            Mode::AbsoluteIndexedX(addr) => {
                let ptr1 = Addr::from(cpu.regs.dbr(), addr);
                let ptr2 = ptr1.wrapping_add(cpu.regs.x(), AddrWrap::Long);
                ModeRead {
                    val: bus.read_word(ptr2, AddrWrap::Long),
                    cycles: 4 + cpu_op16!(cpu) + Self::addr_page_crossed(cpu, ptr1, ptr2),
                    prog_bytes: 3,
                }
            },
            Mode::AbsoluteIndexedY(addr) => {
                let ptr1 = Addr::from(cpu.regs.dbr(), addr);
                let ptr2 = ptr1.wrapping_add(cpu.regs.y(), AddrWrap::Long);
                ModeRead {
                    val: bus.read_word(ptr2, AddrWrap::Long),
                    cycles: 4 + cpu_op16!(cpu) + Self::addr_page_crossed(cpu, ptr1, ptr2),
                    prog_bytes: 3,
                }
            },
            Mode::AbsoluteLong(bank, offset) =>
                ModeRead {
                    val: bus.read_word(Addr::from(bank, offset), AddrWrap::Long),
                    cycles: 5 + cpu_op16!(cpu),
                    prog_bytes: 4,
                },
            Mode::AbsoluteLongIndexed(bank, offset) => {
                let addr = Addr::from(bank, offset)
                    .wrapping_add(cpu.regs.x(), AddrWrap::Long);
                ModeRead {
                    val: bus.read_word(addr, AddrWrap::Long),
                    cycles: 5 + cpu_op16!(cpu),
                    prog_bytes: 4,
                }
            },
            Mode::Accumulator => 
                ModeRead {
                    val: cpu.regs.a(),
                    cycles: 2,
                    prog_bytes: 1,
                },
            Mode::Direct(dir) =>
                ModeRead {
                    val: cpu.read_direct_word(bus, dir, 0),
                    cycles: 3 + cpu_op16!(cpu) + cpu_dl0!(cpu),
                    prog_bytes: 2,
                },
            Mode::DirectIndexedX(dir) =>
                ModeRead {
                    val: cpu.read_direct_word(bus, dir, cpu.regs.x()),
                    cycles: 4 + cpu_op16!(cpu) + cpu_dl0!(cpu),
                    prog_bytes: 2,
                },
            Mode::DirectIndexedY(dir) =>
                ModeRead {
                    val: cpu.read_direct_word(bus, dir, cpu.regs.y()),
                    cycles: 4 + cpu_op16!(cpu) + cpu_dl0!(cpu),
                    prog_bytes: 2,
                },
            Mode::DirectIndexedIndirect(dir) =>
                ModeRead {
                    val: bus.read_word(
                        cpu.read_direct_ptr(bus, dir, cpu.regs.x()), 
                        AddrWrap::Long,
                    ),
                    cycles: 6 + cpu_op16!(cpu) + cpu_dl0!(cpu),
                    prog_bytes: 2,
                },
            Mode::DirectIndirect(dir) => {
                println!("DirectIndirect: {:?}", cpu.read_direct_ptr(bus, dir, 0));
                ModeRead {
                    val: bus.read_word(
                        cpu.read_direct_ptr(bus, dir, 0),
                        AddrWrap::Long,
                    ),
                    cycles: 5 + cpu_op16!(cpu) + cpu_dl0!(cpu),
                    prog_bytes: 2,
                }
            },
            Mode::DirectIndirectIndexed(dir) => {
                let ptr1 = cpu.read_direct_ptr(bus, dir, 0);
                let ptr2 = ptr1.wrapping_add(cpu.regs.y(), AddrWrap::Long);

                ModeRead {
                    val: bus.read_word(ptr2, AddrWrap::Long),
                    cycles: 5 + 
                        cpu_op16!(cpu) + 
                        cpu_dl0!(cpu) + 
                        Self::addr_page_crossed(cpu, ptr1, ptr2),
                    prog_bytes: 2,
                }
            },
            Mode::DirectIndirectLong(offset) => {
                let addr= cpu.read_direct_ptr_long(bus, offset, 0);
                ModeRead {
                    val: bus.read_word(addr, AddrWrap::Long),
                    cycles: 6 + cpu_op16!(cpu) + cpu_dl0!(cpu),
                    prog_bytes: 2,
                }
            },
            Mode::DirectIndirectLongIndexed(offset) => {
                let addr = cpu.read_direct_ptr_long(bus, offset, 0)
                    .wrapping_add(cpu.regs.y(), AddrWrap::Word);
                ModeRead {
                    val: bus.read_word(addr, AddrWrap::Long,),
                    cycles: 6 + cpu_op16!(cpu) + cpu_dl0!(cpu),
                    prog_bytes: 2,
                }
            },
            Mode::Immediate(value) => 
                ModeRead {
                    val: value,
                    cycles: 2 + cpu_op16!(cpu),
                    prog_bytes: 2 + cpu_op16!(cpu),
                },
            Mode::StackRelative(offset) => {
                ModeRead {
                    val: cpu.read_stack_word(bus, offset as u16),
                    cycles: 4 + cpu_op16!(cpu),
                    prog_bytes: 2,
                }
            },
            Mode::StackRelativeIndirectIndexed(offset) => {
                let addr = Addr::from(
                    cpu.regs.dbr(),
                    cpu.read_stack_word(bus, offset as u16),
                );
                let ptr = addr.wrapping_add(cpu.regs.y(), AddrWrap::Word);
                ModeRead {
                    val: bus.read_word(ptr, AddrWrap::Long,),
                    cycles: 7 + cpu_op16!(cpu),
                    prog_bytes: 2,
                }
            },
            _ => panic!("addressing mode does not support read operation"),
        }
    }

    pub fn write(self, cpu: &mut CPU, bus: &mut impl Bus, val: u16) -> ModeWrite {
        if cpu.regs.accum_is_byte() {
            self.write_byte(cpu, bus, val as u8)
        } else {
            self.write_word(cpu, bus, val)
        }
    }

    pub fn jump(self, cpu: &CPU, bus: &impl Bus) -> ModeJump {
        match self {
            Mode::AbsoluteJump(addr) => {
                ModeJump {
                    bank: cpu.regs.pbr(),
                    addr,
                    jmp_cycles: 3,
                    jsr_cycles: 6,
                    prog_bytes: 3,
                }
            },
            Mode::AbsoluteLongJump(bank, addr) => {
                ModeJump {
                    bank,
                    addr,
                    jmp_cycles: 4,
                    jsr_cycles: 8,
                    prog_bytes: 4,
                }
            },
            Mode::AbsoluteIndirectJump(addr) => {
                ModeJump {
                    bank: cpu.regs.pbr(),
                    addr: bus.read_word(
                        Addr::from(0, addr),
                        AddrWrap::Long,
                    ),
                    jmp_cycles: 5,
                    jsr_cycles: 0, // not used
                    prog_bytes: 3,
                }
            },
            Mode::AbsoluteIndirectLongJump(addr) => {
                let indir = Addr::from(0, addr);
                ModeJump {
                    bank: bus.read_byte(indir.wrapping_add(2usize, AddrWrap::Word)),
                    addr: bus.read_word(indir, AddrWrap::Word),
                    jmp_cycles: 6,
                    jsr_cycles: 0, // not used
                    prog_bytes: 3,
                }
            },
            Mode::AbsoluteIndexedIndirectJump(addr) => {
                let indir = Addr::from(0, addr)
                    .wrapping_add(cpu.regs.x(), AddrWrap::Word);
                ModeJump {
                    bank: cpu.regs.pbr(),
                    addr: bus.read_word(indir,AddrWrap::Word),
                    jmp_cycles: 6,
                    jsr_cycles: 8,
                    prog_bytes: 3,
                }
            },
            _ => panic!("addressing mode does not support jump operation"),
        }
    }

    fn write_byte(self, cpu: &mut CPU, bus: &mut impl Bus, val: u8) -> ModeWrite {
        match self {
            Mode::Absolute(addr) => {
                bus.write_byte(
                    Addr::from(cpu.regs.dbr(), addr),
                    val,
                );
                ModeWrite { cycles: 2 }
            },
            Mode::AbsoluteIndexedX(addr) => {
                bus.write_byte(
                    Addr::from(cpu.regs.dbr(), addr)
                        .wrapping_add(cpu.regs.x(), AddrWrap::Long),
                    val,
                );
                ModeWrite { cycles: 2 }
            },
            Mode::Accumulator => {
                cpu.regs.al_set(val);
                ModeWrite { cycles: 0 }
            },
            Mode::Direct(dir) => {
                cpu.write_direct_byte(bus, dir, 0, val);
                ModeWrite { cycles: 2 }
            },
            Mode::DirectIndexedX(dir) => {
                cpu.write_direct_byte(bus, dir, cpu.regs.x(), val);
                ModeWrite { cycles: 2 }
            },
            _ => panic!("addressing mode does not support write operation")
        }
    }

    fn write_word(self, cpu: &mut CPU, bus: &mut impl Bus, val: u16) -> ModeWrite {
        match self {
            Mode::Absolute(addr) => {
                bus.write_word(
                    Addr::from(cpu.regs.dbr(), addr),
                    AddrWrap::Long,
                    val,
                );
                ModeWrite { cycles: 3 }
            },
            Mode::AbsoluteIndexedX(addr) => {
                bus.write_word(
                    Addr::from(cpu.regs.dbr(), addr)
                        .wrapping_add(cpu.regs.x(), AddrWrap::Long),
                    AddrWrap::Long,
                    val,
                );
                ModeWrite { cycles: 3 }
            },
            Mode::Accumulator => {
                cpu.regs.a_set(val);
                ModeWrite { cycles: 0 }
            },
            Mode::Direct(dir) => {
                cpu.write_direct_word(bus, dir, 0, val);
                ModeWrite { cycles: 3 }
            },
            Mode::DirectIndexedX(dir) => {
                cpu.write_direct_word(bus, dir, cpu.regs.x(), val);
                ModeWrite { cycles: 3 }
            },
            _ => panic!("addressing mode does not support write operation")
        }
    }

    fn addr_page_crossed(cpu: &CPU, ptr1: Addr, ptr2: Addr) -> u64 {
        if !ptr1.same_page(ptr2) || cpu.regs.status_flag_is_clear(Flag::X) {
            1
        } else {
            0
        }
    }
}

impl Display for Mode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Mode::Accumulator => write!(f, ""),
            Mode::Absolute(addr) => write!(f, "${:04X}", addr),
            Mode::AbsoluteJump(addr) => write!(f, "${:04X}", addr),
            Mode::AbsoluteIndirectJump(addr) => write!(f, "(${:04X})", addr),
            Mode::AbsoluteIndirectLongJump(addr) => write!(f, "[${:04X}]", addr),
            Mode::AbsoluteIndexedIndirectJump(addr) => write!(f, "(${:04X},X)", addr),
            Mode::AbsoluteLongJump(bank, addr) => write!(f, "${:02X}{:04X}", bank, addr),
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
        ModeRead{val: 0x00AB, cycles: 4, prog_bytes: 3},    // expected
    )]
    #[case::absolute_native_8bit(
        "P.E:0,P.M:1,DBR:12",                               // cpu
        "123456:AB",                                        // bus
        Mode::Absolute(0x3456),                             // addr
        ModeRead{val: 0xAB, cycles: 4, prog_bytes: 3},      // expected
    )]
    #[case::absolute_native_16bit(
        "P.E:0,DBR:12",                                     // cpu
        "123456:CDAB",                                      // bus
        Mode::Absolute(0x3456),                             // addr
        ModeRead{val: 0xABCD, cycles: 5, prog_bytes: 3},    // expected
    )]

    /****************************/
    /* Absolute indexed X : a,X */
    /****************************/
    #[case::absolute_indexed_x_emulated(
        "P.E:1,X:12",                                       // cpu
        "001246:AB",                                        // bus
        Mode::AbsoluteIndexedX(0x1234),                     // addr
        ModeRead{val: 0x00AB, cycles: 4, prog_bytes: 3},    // expected
    )]
    #[case::absolute_indexed_x_native_8bit(
        "P.E:0,P.M:1,P.X:1,DBR:12,X:12",                    // cpu
        "123468:AB",                                        // bus
        Mode::AbsoluteIndexedX(0x3456),                     // addr
        ModeRead{val: 0xAB, cycles: 4, prog_bytes: 3},      // expected
    )]
    #[case::absolute_indexed_x_native_8bit_16bitidx(
        "P.E:0,P.M:1,P.X:0,DBR:12,X:12",                    // cpu
        "123468:AB",                                        // bus
        Mode::AbsoluteIndexedX(0x3456),                     // addr
        ModeRead{val: 0xAB, cycles: 5, prog_bytes: 3},      // expected
    )]
    #[case::absolute_indexed_x_native_16bit(
        "P.E:0,DBR:12,X:12",                                // cpu
        "123468:CDAB",                                      // bus
        Mode::AbsoluteIndexedX(0x3456),                     // addr
        ModeRead{val: 0xABCD, cycles: 6, prog_bytes: 3},    // expected
    )]
    #[case::absolute_indexed_x_native_16bit_8bitidx(
        "P.E:0,P.X:1,DBR:12,X:12",                          // cpu
        "123468:CDAB",                                      // bus
        Mode::AbsoluteIndexedX(0x3456),                     // addr
        ModeRead{val: 0xABCD, cycles: 5, prog_bytes: 3},    // expected
    )]

    /**********************/
    /* Absolute long : al */
    /**********************/
    #[case::absolute_long_8bit(
        "P.E:0,P.M:1",                                      // cpu
        "123456:AB",                                        // bus
        Mode::AbsoluteLong(0x12, 0x3456),                   // addr
        ModeRead{val: 0x00AB, cycles: 5, prog_bytes: 4},    // expected
    )]
    #[case::absolute_long_16bit(
        "P.E:0",                                            // cpu
        "123456:CDAB",                                      // bus
        Mode::AbsoluteLong(0x12, 0x3456),                   // addr
        ModeRead{val: 0xABCD, cycles: 6, prog_bytes: 4},    // expected
    )]

    /********************************/
    /* Absolute long indexed : al,X */
    /********************************/
    #[case::absolute_long_indexed_8bit(
        "P.E:0,P.M:1,X:12",                                 // cpu
        "123468:AB",                                        // bus
        Mode::AbsoluteLongIndexed(0x12, 0x3456),            // addr
        ModeRead{val: 0x00AB, cycles: 5, prog_bytes: 4},    // expected
    )]
    #[case::absolute_long_indexed_16bit(
        "P.E:0,X:12",                                       // cpu
        "123468:CDAB",                                      // bus
        Mode::AbsoluteLongIndexed(0x12, 0x3456),            // addr
        ModeRead{val: 0xABCD, cycles: 6, prog_bytes: 4},    // expected
    )]

    /**************/
    /* Direct : d */
    /**************/
    #[case::direct_emulated(
        "P.E:1,DP:FF10",                                    // cpu
        "00FF30:12",                                        // bus
        Mode::Direct(0x20),                                 // addr
        ModeRead{val: 0x0012, cycles: 4, prog_bytes: 2},    // expected
    )]
    #[case::direct_emulated_dl0(
        "P.E:1,DP:FF00",                                    // cpu
        "00FF20:12",                                        // bus
        Mode::Direct(0x20),                                 // addr
        ModeRead{val: 0x0012, cycles: 3, prog_bytes: 2},    // expected
    )]
    #[case::direct_emulated_page_wrapping(
        "P.E:1,DP:FFF0",                                    // cpu
        "000010:12",                                        // bus
        Mode::Direct(0x20),                                 // addr
        ModeRead{val: 0x0012, cycles: 4, prog_bytes: 2},    // expected
    )]
    #[case::direct_native_8bit(
        "P.E:0,P.M:1,DP:FF10",                              // cpu
        "00FF30:12",                                        // bus
        Mode::Direct(0x20),                                 // addr
        ModeRead{val: 0x0012, cycles: 4, prog_bytes: 2},    // expected
    )]
    #[case::direct_native_8bit_dl0(
        "P.E:0,P.M:1,DP:FF00",                              // cpu
        "00FF20:12",                                        // bus
        Mode::Direct(0x20),                                 // addr
        ModeRead{val: 0x0012, cycles: 3, prog_bytes: 2},    // expected
    )]
    #[case::direct_native_8bit_bank_wrapping(
        "P.E:0,P.M:1,DP:FFF0",                              // cpu
        "000010:12",                                        // bus
        Mode::Direct(0x20),                                 // addr
        ModeRead{val: 0x0012, cycles: 4, prog_bytes: 2},    // expected
    )]
    #[case::direct_native_16bit(
        "P.E:0,P.M:0,DP:FF10",                              // cpu
        "00FF30:3412",                                      // bus
        Mode::Direct(0x20),                                 // addr
        ModeRead{val: 0x1234, cycles: 5, prog_bytes: 2},    // expected
    )]
    #[case::direct_native_16bit_dl0(
        "P.E:0,P.M:0,DP:FF00",                              // cpu
        "00FF20:3412",                                      // bus
        Mode::Direct(0x20),                                 // addr
        ModeRead{val: 0x1234, cycles: 4, prog_bytes: 2},    // expected
    )]
    #[case::direct_native_16bit_wrapping(
        "P.E:0,P.M:0,DP:FFF0",                              // cpu
        "000010:3412",                                      // bus
        Mode::Direct(0x20),                                 // addr
        ModeRead{val: 0x1234, cycles: 5, prog_bytes: 2},    // expected
    )]

    /**************************/
    /* Direct indexed X : d,X */
    /**************************/
    #[case::direct_indexed_x_emulated(
        "P.E:1,DP:FF10,X:12",                               // cpu
        "00FF42:12",                                        // bus
        Mode::DirectIndexedX(0x20),                         // addr
        ModeRead{val: 0x0012, cycles: 5, prog_bytes: 2},    // expected
    )]
    #[case::direct_indexed_x_emulated_dl0(
        "P.E:1,DP:FF00,X:12",                               // cpu
        "00FF32:12",                                        // bus
        Mode::DirectIndexedX(0x20),                         // addr
        ModeRead{val: 0x0012, cycles: 4, prog_bytes: 2},    // expected
    )]
    #[case::direct_indexed_x_emulated_page_wrapping(
        "P.E:1,DP:FFF0,X:12",                               // cpu
        "000022:12",                                        // bus
        Mode::DirectIndexedX(0x20),                         // addr
        ModeRead{val: 0x0012, cycles: 5, prog_bytes: 2},    // expected
    )]
    #[case::direct_indexed_x_native_8bit(
        "P.E:0,P.M:1,DP:FF10,X:12",                         // cpu
        "00FF42:12",                                        // bus
        Mode::DirectIndexedX(0x20),                         // addr
        ModeRead{val: 0x0012, cycles: 5, prog_bytes: 2},    // expected
    )]
    #[case::direct_indexed_x_native_8bit_dl0(
        "P.E:0,P.M:1,DP:FF00,X:12",                         // cpu
        "00FF32:12",                                        // bus
        Mode::DirectIndexedX(0x20),                         // addr
        ModeRead{val: 0x0012, cycles: 4, prog_bytes: 2},    // expected
    )]
    #[case::direct_indexed_x_native_8bit_bank_wrapping(
        "P.E:0,P.M:1,DP:FFF0,X:12",                         // cpu
        "000022:12",                                        // bus
        Mode::DirectIndexedX(0x20),                         // addr
        ModeRead{val: 0x0012, cycles: 5, prog_bytes: 2},    // expected
    )]
    #[case::direct_indexed_x_native_16bit(
        "P.E:0,P.M:0,DP:FF10,X:12",                         // cpu
        "00FF42:3412",                                      // bus
        Mode::DirectIndexedX(0x20),                         // addr
        ModeRead{val: 0x1234, cycles: 6, prog_bytes: 2},    // expected
    )]
    #[case::direct_indexed_x_native_16bit_dl0(
        "P.E:0,P.M:0,DP:FF00,X:12",                         // cpu
        "00FF32:3412",                                      // bus
        Mode::DirectIndexedX(0x20),                         // addr
        ModeRead{val: 0x1234, cycles: 5, prog_bytes: 2},    // expected
    )]
    #[case::direct_indexed_x_native_16bit_wrapping(
        "P.E:0,P.M:0,DP:FFF0,X:12",                         // cpu
        "000022:3412",                                      // bus
        Mode::DirectIndexedX(0x20),                         // addr
        ModeRead{val: 0x1234, cycles: 6, prog_bytes: 2},    // expected
    )]

    /**************************/
    /* Direct indexed Y : d,Y */
    /**************************/
    #[case::direct_indexed_y_emulated(
        "P.E:1,DP:FF10,Y:12",                               // cpu
        "00FF42:12",                                        // bus
        Mode::DirectIndexedY(0x20),                         // addr
        ModeRead{val: 0x0012, cycles: 5, prog_bytes: 2},    // expected
    )]
    #[case::direct_indexed_y_emulated_dl0(
        "P.E:1,DP:FF00,Y:12",                               // cpu
        "00FF32:12",                                        // bus
        Mode::DirectIndexedY(0x20),                         // addr
        ModeRead{val: 0x0012, cycles: 4, prog_bytes: 2},    // expected
    )]
    #[case::direct_indexed_y_emulated_page_wrapping(
        "P.E:1,DP:FFF0,Y:12",                               // cpu
        "000022:12",                                        // bus
        Mode::DirectIndexedY(0x20),                         // addr
        ModeRead{val: 0x0012, cycles: 5, prog_bytes: 2},    // expected
    )]
    #[case::direct_indexed_y_native_8bit(
        "P.E:0,P.M:1,DP:FF10,Y:12",                         // cpu
        "00FF42:12",                                        // bus
        Mode::DirectIndexedY(0x20),                         // addr
        ModeRead{val: 0x0012, cycles: 5, prog_bytes: 2},    // expected
    )]
    #[case::direct_indexed_y_native_8bit_dl0(
        "P.E:0,P.M:1,DP:FF00,Y:12",                         // cpu
        "00FF32:12",                                        // bus
        Mode::DirectIndexedY(0x20),                         // addr
        ModeRead{val: 0x0012, cycles: 4, prog_bytes: 2},    // expected
    )]
    #[case::direct_indexed_y_native_8bit_bank_wrapping(
        "P.E:0,P.M:1,DP:FFF0,Y:12",                         // cpu
        "000022:12",                                        // bus
        Mode::DirectIndexedY(0x20),                         // addr
        ModeRead{val: 0x0012, cycles: 5, prog_bytes: 2},    // expected
    )]
    #[case::direct_indexed_y_native_16bit(
        "P.E:0,P.M:0,DP:FF10,Y:12",                         // cpu
        "00FF42:3412",                                      // bus
        Mode::DirectIndexedY(0x20),                         // addr
        ModeRead{val: 0x1234, cycles: 6, prog_bytes: 2},    // expected
    )]
    #[case::direct_indexed_y_native_16bit_dl0(
        "P.E:0,P.M:0,DP:FF00,Y:12",                         // cpu
        "00FF32:3412",                                      // bus
        Mode::DirectIndexedY(0x20),                         // addr
        ModeRead{val: 0x1234, cycles: 5, prog_bytes: 2},    // expected
    )]
    #[case::direct_indexed_y_native_16bit_wrapping(
        "P.E:0,P.M:0,DP:FFF0,Y:12",                         // cpu
        "000022:3412",                                      // bus
        Mode::DirectIndexedY(0x20),                         // addr
        ModeRead{val: 0x1234, cycles: 6, prog_bytes: 2},    // expected
    )]

    /***********************************/
    /* Direct indexed indirect : (d,X) */
    /***********************************/
    #[case::direct_indexed_indirect_emulated(
        "P.E:1,DP:1040,X:12",                               // cpu
        "001056:8040,004080:3412",                          // bus
        Mode::DirectIndexedIndirect(0x04),                  // addr
        ModeRead{val: 0x1234, cycles: 7, prog_bytes: 2},    // expected
    )]
    #[case::direct_indexed_indirect_emulated_dl0(
        "P.E:1,DP:1000,X:12",                               // cpu
        "001016:8040,004080:3412",                          // bus
        Mode::DirectIndexedIndirect(0x04),                  // addr
        ModeRead{val: 0x1234, cycles: 6, prog_bytes: 2},    // expected
    )]
    #[case::direct_indexed_indirect_emulated_page_wrapping(
        "P.E:1,DP:1000,X:FF",                               // cpu
        "001003:8040,004080:3412",                          // bus
        Mode::DirectIndexedIndirect(0x04),                  // addr
        ModeRead{val: 0x1234, cycles: 6, prog_bytes: 2},    // expected
    )]
    #[case::direct_indexed_indirect_emulated_bank_wrapping(
        "P.E:1,DP:FFF0,X:20",                               // cpu
        "000014:8040,004080:3412",                          // bus
        Mode::DirectIndexedIndirect(0x04),                  // addr
        ModeRead{val: 0x1234, cycles: 7, prog_bytes: 2},    // expected
    )]
    #[case::direct_indexed_indirect_native_16bit(
        "P.E:0,P.M:0,DP:1040,X:12",                         // cpu
        "001056:8040,004080:3412",                          // bus
        Mode::DirectIndexedIndirect(0x04),                  // addr
        ModeRead{val: 0x1234, cycles: 8, prog_bytes: 2},    // expected
    )]
    #[case::direct_indexed_indirect_native_16bit_bank_wrapping(
        "P.E:0,DP:FFF0,X:20",                               // cpu
        "000014:8040,004080:3412",                          // bus
        Mode::DirectIndexedIndirect(0x04),                  // addr
        ModeRead{val: 0x1234, cycles: 8, prog_bytes: 2},    // expected
    )]

    /*************************/
    /* Direct indirect : (d) */
    /*************************/
    #[case::direct_indirect_emulated(
        "P.E:1,DP:1040",                                    // cpu
        "001044:8040,004080:3412",                          // bus
        Mode::DirectIndirect(0x04),                         // addr
        ModeRead{val: 0x1234, cycles: 6, prog_bytes: 2},    // expected
    )]
    #[case::direct_indirect_emulated_dl0(
        "P.E:1,DP:1000",                                    // cpu
        "001004:8040,004080:3412",                          // bus
        Mode::DirectIndirect(0x04),                         // addr
        ModeRead{val: 0x1234, cycles: 5, prog_bytes: 2},    // expected
    )]
    #[case::direct_indirect_emulated_page_wrapping(
        "P.E:1,DP:1000",                                    // cpu
        "001000:40,0010FF:80,004080:3412",                  // bus
        Mode::DirectIndirect(0xFF),                         // addr
        ModeRead{val: 0x1234, cycles: 5, prog_bytes: 2},    // expected
    )]
    #[case::direct_indirect_emulated_bank_wrapping(
        "P.E:1,DP:FFF0",                                    // cpu
        "000014:8040,004080:3412",                          // bus
        Mode::DirectIndirect(0x24),                         // addr
        ModeRead{val: 0x1234, cycles: 6, prog_bytes: 2},    // expected
    )]
    #[case::direct_indirect_native_8bit(
        "P.E:0,P.M:1,DP:1040",                              // cpu
        "001044:8040,004080:3412",                          // bus
        Mode::DirectIndirect(0x04),                         // addr
        ModeRead{val: 0x1234, cycles: 6, prog_bytes: 2},    // expected
    )]
    #[case::direct_indirect_native_8bit_dl0(
        "P.E:0,P.M:1,DP:1000",                              // cpu
        "001004:8040,004080:3412",                          // bus
        Mode::DirectIndirect(0x04),                         // addr
        ModeRead{val: 0x1234, cycles: 5, prog_bytes: 2},    // expected
    )]
    #[case::direct_indirect_native_16bit(
        "P.E:0,P.M:0,DP:1040",                              // cpu
        "001044:8040,004080:3412",                          // bus
        Mode::DirectIndirect(0x04),                         // addr
        ModeRead{val: 0x1234, cycles: 7, prog_bytes: 2},    // expected
    )]
    #[case::direct_indirect_native_16bit_dl0(
        "P.E:0,P.M:0,DP:1000",                              // cpu
        "001004:8040,004080:3412",                          // bus
        Mode::DirectIndirect(0x04),                         // addr
        ModeRead{val: 0x1234, cycles: 6, prog_bytes: 2},    // expected
    )]
    #[case::direct_indirect_native_16bit_bank_wrapping(
        "P.E:0,DP:FFF0",                                    // cpu
        "000014:8040,004080:3412",                          // bus
        Mode::DirectIndirect(0x24),                         // addr
        ModeRead{val: 0x1234, cycles: 7, prog_bytes: 2},    // expected
    )]

    /***********************************/
    /* Direct indirect indexed : (d),Y */
    /***********************************/
    #[case::direct_indirect_indexed_emulated(
        "P.E:1,DP:1040,Y:12",                               // cpu
        "001044:8040,004092:3412",                          // bus
        Mode::DirectIndirectIndexed(0x04),                  // addr
        ModeRead{val: 0x1234, cycles: 6, prog_bytes: 2},    // expected
    )]
    #[case::direct_indirect_indexed_emulated_dl0(
        "P.E:1,DP:1000,Y:12",                               // cpu
        "001004:8040,004092:3412",                          // bus
        Mode::DirectIndirectIndexed(0x04),                  // addr
        ModeRead{val: 0x1234, cycles: 5, prog_bytes: 2},    // expected
    )]
    #[case::direct_indirect_indexed_emulated_page_wrapping(
        "P.E:1,DP:1000,Y:12",                               // cpu
        "001000:40,0010FF:80,004092:3412",                          // bus
        Mode::DirectIndirectIndexed(0xFF),                  // addr
        ModeRead{val: 0x1234, cycles: 5, prog_bytes: 2},    // expected
    )]
    #[case::direct_indirect_indexed_emulated_bank_wrapping(
        "P.E:1,DP:FFF0,Y:12",                               // cpu
        "000014:8040,004092:3412",                          // bus
        Mode::DirectIndirectIndexed(0x24),                  // addr
        ModeRead{val: 0x1234, cycles: 6, prog_bytes: 2},    // expected
    )]
    #[case::direct_indirect_indexed_native_16bit(
        "P.E:0,P.M:0,P.X:0,DP:1040,Y:12",                   // cpu
        "001044:8040,004092:3412",                          // bus
        Mode::DirectIndirectIndexed(0x04),                  // addr
        ModeRead{val: 0x1234, cycles: 8, prog_bytes: 2},    // expected
    )]
    #[case::direct_indirect_indexed_native_16bit_8bitidx(
        "P.E:0,P.M:0,P.X:1,DP:1040,Y:12",                   // cpu
        "001044:8040,004092:3412",                          // bus
        Mode::DirectIndirectIndexed(0x04),                  // addr
        ModeRead{val: 0x1234, cycles: 7, prog_bytes: 2},    // expected
    )]
    #[case::direct_indirect_indexed_native_16bit_8bitidx_wrapping(
        "P.E:0,P.M:0,P.X:1,DP:1040,Y:22",                   // cpu
        "001044:F040,004112:3412",                          // bus
        Mode::DirectIndirectIndexed(0x04),                  // addr
        ModeRead{val: 0x1234, cycles: 8, prog_bytes: 2},    // expected
    )]
    #[case::direct_indirect_indexed_native_16bit_bank_wrapping(
        "P.E:0,DP:FFF0,Y:12",                               // cpu
        "000014:8040,004092:3412",                          // bus
        Mode::DirectIndirectIndexed(0x24),                  // addr
        ModeRead{val: 0x1234, cycles: 8, prog_bytes: 2},    // expected
    )]

    /******************************/
    /* Direct indirect long : [d] */
    /******************************/
    #[case::direct_indirect_long_8bit(
        "P.E:0,P.M:1,DP:FF10",                              // cpu
        "00FF30:563412,123456:AB",                          // bus
        Mode::DirectIndirectLong(0x20),                     // addr
        ModeRead{val: 0xAB, cycles: 7, prog_bytes: 2},      // expected
    )]
    #[case::direct_indirect_long_8bit_dl0(
        "P.E:0,P.M:1,DP:FF00",                              // cpu
        "00FF20:563412,123456:AB",                          // bus
        Mode::DirectIndirectLong(0x20),                     // addr
        ModeRead{val: 0xAB, cycles: 6, prog_bytes: 2},      // expected
    )]
    #[case::direct_indirect_long_16bit(
        "P.E:0,P.M:0,DP:FF10",                              // cpu
        "00FF30:563412,123456:CDAB",                        // bus
        Mode::DirectIndirectLong(0x20),                     // addr
        ModeRead{val: 0xABCD, cycles: 8, prog_bytes: 2},    // expected
    )]
    #[case::direct_indirect_long_16bit_dl0(
        "P.E:0,P.M:0,DP:FF00",                              // cpu
        "00FF20:563412,123456:CDAB",                        // bus
        Mode::DirectIndirectLong(0x20),                     // addr
        ModeRead{val: 0xABCD, cycles: 7, prog_bytes: 2},    // expected
    )]
    #[case::direct_indirect_long_bank_wrapping(
        "P.E:0,P.M:0,DP:FFF0",                              // cpu
        "000010:FFFF12,12FFFF:CDAB",                        // bus
        Mode::DirectIndirectLong(0x20),                     // addr
        ModeRead{val: 0xABCD, cycles: 8, prog_bytes: 2},    // expected
    )]

    /****************************************/
    /* Direct indirect long indexed : [d],X */
    /****************************************/
    #[case::direct_indirect_long_indexed_8bit(
        "P.E:0,P.M:1,DP:FF10,Y:12",                         // cpu
        "00FF30:563412,123468:AB",                          // bus
        Mode::DirectIndirectLongIndexed(0x20),              // addr
        ModeRead{val: 0xAB, cycles: 7, prog_bytes: 2},      // expected
    )]
    #[case::direct_indirect_long_indexed_8bit_dl0(
        "P.E:0,P.M:1,DP:FF00,Y:12",                         // cpu
        "00FF20:563412,123468:AB",                          // bus
        Mode::DirectIndirectLongIndexed(0x20),              // addr
        ModeRead{val: 0xAB, cycles: 6, prog_bytes: 2},      // expected
    )]
    #[case::direct_indirect_long_indexed_16bit(
        "P.E:0,P.M:0,DP:FF10,Y:12",                         // cpu
        "00FF30:563412,123468:CDAB",                        // bus
        Mode::DirectIndirectLongIndexed(0x20),              // addr
        ModeRead{val: 0xABCD, cycles: 8, prog_bytes: 2},    // expected
    )]
    #[case::direct_indirect_long_indexed_16bit_dl0(
        "P.E:0,P.M:0,DP:FF00,Y:12",                         // cpu
        "00FF20:563412,123468:CDAB",                        // bus
        Mode::DirectIndirectLongIndexed(0x20),              // addr
        ModeRead{val: 0xABCD, cycles: 7, prog_bytes: 2},    // expected
    )]
    #[case::direct_indirect_long_indexed_bank_wrapping(
        "P.E:0,P.M:0,DP:FFF0,Y:0F",                         // cpu
        "000010:F0FF12,12FFFF:CDAB",                        // bus
        Mode::DirectIndirectLongIndexed(0x20),              // addr
        ModeRead{val: 0xABCD, cycles: 8, prog_bytes: 2},    // expected
    )]

    /*******************/
    /* Inmmediate : #i */
    /*******************/
    #[case::immediate_8bit(
        "P.E:1",                                            // cpu
        "",                                                 // bus
        Mode::Immediate(0x00AB),                            // addr
        ModeRead{val: 0x00AB, cycles: 2, prog_bytes: 2},    // expected
    )]
    #[case::immediate_16bit(
        "P.E:0",                                            // cpu
        "",                                                 // bus
        Mode::Immediate(0x00AB),                            // addr
        ModeRead{val: 0x00AB, cycles: 3, prog_bytes: 3},    // expected
    )]

    /**************************/
    /* Stack relative : (d,S) */
    /**************************/
    #[case::stack_relative_emulated(
        "P.E:1,SP:FFF0",                                    // cpu
        "0001F2:12",                                        // bus
        Mode::StackRelative(0x02),                          // addr
        ModeRead{val: 0x12, cycles: 4, prog_bytes: 2},      // expected
    )]
    #[case::stack_relative_native_8bit(
        "P.E:0,P.M:1,SP:FFF0",                              // cpu
        "00FFF2:12",                                        // bus
        Mode::StackRelative(0x02),                          // addr
        ModeRead{val: 0x12, cycles: 4, prog_bytes: 2},      // expected
    )]
    #[case::stack_relative_native_16bit(
        "P.E:0,SP:FFF0",                                    // cpu
        "00FFF2:3412",                                      // bus
        Mode::StackRelative(0x02),                          // addr
        ModeRead{val: 0x1234, cycles: 5, prog_bytes: 2},    // expected
    )]
    #[case::stack_relative_bank_wrapping(
        "P.E:0,SP:FFF0",                                    // cpu
        "000002:3412",                                      // bus
        Mode::StackRelative(0x12),                          // addr
        ModeRead{val: 0x1234, cycles: 5, prog_bytes: 2},    // expected
    )]

    /*********************************************/
    /* Stack relative indirect indexed : (d,S),Y */
    /*********************************************/
    #[case::stack_relative_indirect_indexed_emulated(
        "P.E:1,SP:FFF0,Y:08",                               // cpu
        "0001F2:8040,004088:12",                            // bus
        Mode::StackRelativeIndirectIndexed(0x02),           // addr
        ModeRead{val: 0x12, cycles: 7, prog_bytes: 2},      // expected
    )]
    #[case::stack_relative_indirect_indexed_native_8bit(
        "P.E:0,P.M:1,SP:FFF0,Y:08",                         // cpu
        "00FFF2:8040,004088:12",                            // bus
        Mode::StackRelativeIndirectIndexed(0x02),           // addr
        ModeRead{val: 0x12, cycles: 7, prog_bytes: 2},      // expected
    )]
    #[case::stack_relative_indirect_indexed_native_16bit(
        "P.E:0,SP:FFF0,Y:08",                               // cpu
        "00FFF2:8040,004088:3412",                          // bus
        Mode::StackRelativeIndirectIndexed(0x02),           // addr
        ModeRead{val: 0x1234, cycles: 8, prog_bytes: 2},    // expected
    )]
    #[case::stack_relative_indirect_indexed_bank_wrapping(
        "P.E:0,SP:FFF0,Y:08",                               // cpu
        "000002:8040,004088:3412",                          // bus
        Mode::StackRelativeIndirectIndexed(0x12),           // addr
        ModeRead{val: 0x1234, cycles: 8, prog_bytes: 2},    // expected
    )]
    fn test_read(
        #[case] mut cpu: CPU,
        #[case] bus: bus::Fake,
        #[case] addr: Mode,
        #[case] expected: ModeRead,
    ) {
        let rd = addr.read(&mut cpu, &bus);
        assert_eq!(rd.val, expected.val);
        assert_eq!(rd.cycles, expected.cycles);
        assert_eq!(rd.prog_bytes, expected.prog_bytes);
    }

    #[rstest]
    #[case::absolute_8bit(
        "P.E:1", 
        Mode::Absolute(0x5678),
        0x1234,
        0x34,
        ModeWrite{cycles: 2},
    )]
    #[case::absolute_16bit(
        "P.E:0", 
        Mode::Absolute(0x5678),
        0x1234,
        0x1234,
        ModeWrite{cycles: 3},
    )]
    #[case::absolute_indexed_x_8bit(
        "P.E:1,X:AB", 
        Mode::AbsoluteIndexedX(0x5678),
        0x1234,
        0x34,
        ModeWrite{cycles: 2},
    )]
    #[case::absolute_indexed_x_16bit(
        "P.E:0,X:ABCD", 
        Mode::AbsoluteIndexedX(0x5678),
        0x1234,
        0x1234,
        ModeWrite{cycles: 3},
    )]
    #[case::direct_8bit(
        "P.E:1", 
        Mode::Direct(0x56),
        0x1234,
        0x34,
        ModeWrite{cycles: 2},
    )]
    #[case::direct_16bit(
        "P.E:0", 
        Mode::Direct(0x56),
        0x1234,
        0x1234,
        ModeWrite{cycles: 3},
    )]
    #[case::direct_indexed_x_8bit(
        "P.E:1,X:AB", 
        Mode::DirectIndexedX(0x56),
        0x1234,
        0x34,
        ModeWrite{cycles: 2},
    )]
    #[case::direct_indexed_x_16bit(
        "P.E:0,X:ABCD", 
        Mode::DirectIndexedX(0x56),
        0x1234,
        0x1234,
        ModeWrite{cycles: 3},
    )]
    fn test_write(
        #[case] mut cpu: CPU,
        #[case] addr: Mode,
        #[case] input: u16,
        #[case] output: u16,
        #[case] expected: ModeWrite,
    ) {
        let mut bus = bus::Fake::new();
        let write = addr.write(&mut cpu, &mut bus, input);
        let read = addr.read(&mut cpu, &bus);

        assert_eq!(read.val, output);
        assert_eq!(write.cycles, expected.cycles);
    }

    #[rstest]
    #[case::absolute(
        "PBR:A0",                                                       // cpu
        "",                                                             // bus
        Mode::AbsoluteJump(0x1234),                                     // addr
        ModeJump{                       // expected
            bank: 0xA0, 
            addr: 0x1234, 
            jmp_cycles: 3, 
            jsr_cycles: 6, 
            prog_bytes: 3,
        },
    )]
    #[case::absolute_long(
        "PBR:A0",                                       // cpu
        "",                                             // bus
        Mode::AbsoluteLongJump(0xB0, 0x1234),           // addr
        ModeJump{                                       // expected
            bank: 0xB0, 
            addr: 0x1234, 
            jmp_cycles: 4, 
            jsr_cycles: 8, 
            prog_bytes: 4,
        },
    )]
    #[case::absolute_indirect(
        "PBR:A0",                                       // cpu
        "001234:CDAB",                                  // bus
        Mode::AbsoluteIndirectJump(0x1234),             // addr
        ModeJump{                                       // expected
            bank: 0xA0, 
            addr: 0xABCD, 
            jmp_cycles: 5, 
            jsr_cycles: 0, 
            prog_bytes: 3
        },
    )]
    #[case::absolute_indexed_indirect(
        "PBR:A0,X:02",                                  // cpu
        "001236:CDAB",                                  // bus
        Mode::AbsoluteIndexedIndirectJump(0x1234),      // addr
        ModeJump{                                       // expected
            bank: 0xA0, 
            addr: 0xABCD, 
            jmp_cycles: 6, 
            jsr_cycles: 8, 
            prog_bytes: 3
        },
    )]
    #[case::absolute_indirect_long(
        "PBR:A0",                                       // cpu
        "001234:CDABB0",                                // bus
        Mode::AbsoluteIndirectLongJump(0x1234),         // addr
        ModeJump{                                       // expected
            bank: 0xB0, 
            addr: 0xABCD, 
            jmp_cycles: 6, 
            jsr_cycles: 0, 
            prog_bytes: 3
        },
    )]
    fn test_jump(
        #[case] mut cpu: CPU,
        #[case] bus: bus::Fake,
        #[case] addr: Mode,
        #[case] expected: ModeJump,
    ) {
        let jp = addr.jump(&mut cpu, &bus);
        assert_eq!(jp.bank, expected.bank);
        assert_eq!(jp.addr, expected.addr);
        assert_eq!(jp.jmp_cycles, expected.jmp_cycles);
        assert_eq!(jp.prog_bytes, expected.prog_bytes);
    }

}
