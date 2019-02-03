// Evaluate the given expression in the context of the CPU
macro_rules! cpu_eval {

    // Assign to register A
    ($cpu:expr, A <- $($rhs:tt)+) => ({
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.regs_mut().set_a(val);
        val
    });

    // Assign and set flags
    ($cpu:expr, F +<- ($($flags:tt)+)) => ({
        let mut flags = cpu_eval!($cpu, F);
        flags = flags_apply!(flags, $($flags)+);
        cpu_eval!($cpu, F <- flags);
    });

    // Assign to register F
    ($cpu:expr, F <- $($rhs:tt)+) => ({
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.regs_mut().set_flags(val);
        val
    });

    // Assign to register B
    ($cpu:expr, B <- $($rhs:tt)+) => ({
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.regs_mut().set_b(val);
        val
    });

    // Assign to register C
    ($cpu:expr, C <- $($rhs:tt)+) => ({
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.regs_mut().set_c(val);
        val
    });

    // Assign to register D
    ($cpu:expr, D <- $($rhs:tt)+) => ({
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.regs_mut().set_d(val);
        val
    });

    // Assign to register E
    ($cpu:expr, E <- $($rhs:tt)+) => ({
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.regs_mut().set_e(val);
        val
    });

    // Assign to register H
    ($cpu:expr, H <- $($rhs:tt)+) => ({
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.regs_mut().set_h(val);
        val
    });

    // Assign to register L
    ($cpu:expr, L <- $($rhs:tt)+) => ({
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.regs_mut().set_l(val);
        val
    });

    // Swap AF and AF'
    ($cpu:expr, AF <-> AF_) => { $cpu.regs_mut().swap_af() };

    // Swap BC and BC'
    ($cpu:expr, BC <-> BC_) => { $cpu.regs_mut().swap_bc() };

    // Swap DE and DE'
    ($cpu:expr, DE <-> DE_) => { $cpu.regs_mut().swap_de() };

    // Swap HL and HL'
    ($cpu:expr, HL <-> HL_) => { $cpu.regs_mut().swap_hl() };

    // Assign to AF register
    ($cpu:expr, AF <- $($rhs:tt)+) => ({
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.regs_mut().set_af(val);
        val
    });

    // Assign to AF' register
    ($cpu:expr, AF_ <- $($rhs:tt)+) => ({
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.regs_mut().set_af_(val);
        val
    });

    // Assign to BC register
    ($cpu:expr, BC <- $($rhs:tt)+) => ({
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.regs_mut().set_bc(val);
        val
    });

    // Assign to BC_ register
    ($cpu:expr, BC_ <- $($rhs:tt)+) => ({
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.regs_mut().set_bc_(val);
        val
    });

    // Assign to DE register
    ($cpu:expr, DE <- $($rhs:tt)+) => ({
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.regs_mut().set_de(val);
        val
    });

    // Assign to DE_ register
    ($cpu:expr, DE_ <- $($rhs:tt)+) => ({
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.regs_mut().set_de_(val);
        val
    });

    // Assign to HL register
    ($cpu:expr, HL <- $($rhs:tt)+) => ({
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.regs_mut().set_hl(val);
        val
    });

    // Assign to HL_ register
    ($cpu:expr, HL_ <- $($rhs:tt)+) => ({
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.regs_mut().set_hl_(val);
        val
    });

    // Assign to SP register
    ($cpu:expr, SP <- $($rhs:tt)+) => ({
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.regs_mut().set_sp(val);
        val
    });

    // Assign and increment PC register using 8-bits
    ($cpu:expr, PC +<- $($rhs:tt)+) => ({
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.regs_mut().inc_pc8(val)
    });

    // Assign and increment PC register using 16 bits
    ($cpu:expr, PC ++<- $($rhs:tt)+) => ({
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.regs_mut().inc_pc(val)
    });

    // Assign and increment SP register using 16 bits
    ($cpu:expr, SP ++<- $($rhs:tt)+) => ({
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.regs_mut().inc_sp(val)
    });

    // Assign and decrement SP register using 16 bits
    ($cpu:expr, SP --<- $($rhs:tt)+) => ({
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.regs_mut().dec_sp(val)
    });

    // Assign to PC register
    ($cpu:expr, PC <- $($rhs:tt)+) => ({
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.regs_mut().set_pc(val);
        val
    });

    // Increment PC by one
    ($cpu:expr, PC++) => { $cpu.regs_mut().inc_pc(1) };

    // Indirect write access of bytes
    ($cpu:expr, (*$lhs:tt) <- $($rhs:tt)+) => ({
        let addr = cpu_eval!($cpu, $lhs);
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.mem().write_to(addr, val);
        val
    });

    // Indirect write access of words
    ($cpu:expr, (**$lhs:tt) <- $($rhs:tt)+) => ({
        let addr = cpu_eval!($cpu, $lhs);
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.mem().write_word_to_mem::<LittleEndian>(addr, val) ;
        val
    });

    // IO port write
    ($cpu:expr, IO($addr:expr) <- $($rhs:tt)+) => ({
        let val = cpu_eval!($cpu, $($rhs)+);
        $cpu.io().write_to($addr, val);
        val
    });

    // Add operator
    ($cpu:expr, ($a:tt + $b:tt)) => { cpu_eval!($cpu, $a) + cpu_eval!($cpu, $b) };

    // Read registers
    ($cpu:expr, A) => { $cpu.regs().a() };
    ($cpu:expr, F) => { $cpu.regs().flags() };
    ($cpu:expr, B) => { $cpu.regs().b() };
    ($cpu:expr, C) => { $cpu.regs().c() };
    ($cpu:expr, D) => { $cpu.regs().d() };
    ($cpu:expr, E) => { $cpu.regs().e() };
    ($cpu:expr, H) => { $cpu.regs().h() };
    ($cpu:expr, L) => { $cpu.regs().l() };
    ($cpu:expr, AF) => { $cpu.regs().af() };
    ($cpu:expr, AF_) => { $cpu.regs().af_() };
    ($cpu:expr, BC) => { $cpu.regs().bc() };
    ($cpu:expr, BC_) => { $cpu.regs().bc_() };
    ($cpu:expr, DE) => { $cpu.regs().de() };
    ($cpu:expr, DE_) => { $cpu.regs().de_() };
    ($cpu:expr, HL) => { $cpu.regs().hl() };
    ($cpu:expr, HL_) => { $cpu.regs().hl_() };
    ($cpu:expr, SP) => { $cpu.regs().sp() };
    ($cpu:expr, PC) => { $cpu.regs().pc() };
    ($cpu:expr, n) => { cpu_eval!($cpu, (*(PC+1))) };
    ($cpu:expr, nn) => { cpu_eval!($cpu, (**(PC+1))) };

    // Read single flag
    ($cpu:expr, F[$f:ident]) => ({
        let flags = cpu_eval!($cpu, F);
        flag!(flags, $f)
    });

    // Indirect read access of bytes
    ($cpu:expr, (*$val:tt)) => ({
        let addr = cpu_eval!($cpu, $val);
        $cpu.mem().read_from(addr)
    });

    // Indirect read access of words
    ($cpu:expr, (**$val:tt)) => ({
        let addr = cpu_eval!($cpu, $val);
        $cpu.mem().read_word_from_mem::<LittleEndian>(addr)
    });

    // Read IO port
    ($cpu:expr, IO($addr:expr)) => ({
        $cpu.io().read_from($addr)
    });

    ($cpu:expr, L8) => ({
        let addr = cpu_eval!($cpu, PC) + 1;
        cpu_eval!($cpu, (addr))
    });

    ($cpu:expr, L16) => ({
        let addr = cpu_eval!($cpu, PC) + 1;
        cpu_eval!($cpu, (addr): u16)
    });

    ($cpu:expr, $val:expr) => { $val };
}
