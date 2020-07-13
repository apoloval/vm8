mod reg;

#[derive(Default)]
pub struct Stats {
  total_inst: usize,
  total_cycles: usize,
}

impl Stats {
  pub fn step_over(&mut self, cycles: usize) {
    self.total_inst += 1;
    self.total_cycles += cycles;
  }
}

pub struct CPU {
  pub regs: reg::Bank,
  pub stats: Stats,
}

impl CPU {
  pub fn new() -> CPU {
    CPU{
      regs: reg::Bank::default(),
      stats: Stats::default(),
    }
  }
}

macro_rules! eval {
  // Assignments to 8-bit registers
  ($cpu:expr, A <- $( $val:tt )+) => { $cpu.regs.set_a(eval!($cpu, $( $val )+)) };
  ($cpu:expr, F <- $( $val:tt )+) => { $cpu.regs.set_f(eval!($cpu, $( $val )+)) };
  ($cpu:expr, B <- $( $val:tt )+) => { $cpu.regs.set_b(eval!($cpu, $( $val )+))};
  ($cpu:expr, C <- $( $val:tt )+) => { $cpu.regs.set_c(eval!($cpu, $( $val )+))};
  ($cpu:expr, D <- $( $val:tt )+) => { $cpu.regs.set_d(eval!($cpu, $( $val )+))};
  ($cpu:expr, E <- $( $val:tt )+) => { $cpu.regs.set_e(eval!($cpu, $( $val )+))};
  ($cpu:expr, H <- $( $val:tt )+) => { $cpu.regs.set_h(eval!($cpu, $( $val )+))};
  ($cpu:expr, L <- $( $val:tt )+) => { $cpu.regs.set_l(eval!($cpu, $( $val )+))};

  // Assignments to 16-bit registers
  ($cpu:expr, AF <- $( $val:tt )+) => { $cpu.regs.set_af(eval!($cpu, $( $val )+))};
  ($cpu:expr, BC <- $( $val:tt )+) => { $cpu.regs.set_bc(eval!($cpu, $( $val )+))};
  ($cpu:expr, DE <- $( $val:tt )+) => { $cpu.regs.set_de(eval!($cpu, $( $val )+))};
  ($cpu:expr, HL <- $( $val:tt )+) => { $cpu.regs.set_hl(eval!($cpu, $( $val )+))};
  ($cpu:expr, PC <- $( $val:tt )+) => { $cpu.regs.set_pc(eval!($cpu, $( $val )+))};
  ($cpu:expr, SP <- $( $val:tt )+) => { $cpu.regs.set_sp(eval!($cpu, $( $val )+))};

  // Aritmethic-logic expressions
  ($cpu:expr, $a:tt + $( $b:tt )+)  => { eval!($cpu, $a) + eval!($cpu, $( $b )+) };
  ($cpu:expr, $a:tt - $( $b:tt )+)  => { eval!($cpu, $a) - eval!($cpu, $( $b )+) };
  ($cpu:expr, $a:tt & $( $b:tt )+)  => { eval!($cpu, $a) & eval!($cpu, $( $b )+) };
  ($cpu:expr, $a:tt | $( $b:tt )+)  => { eval!($cpu, $a) | eval!($cpu, $( $b )+) };

  // 8-bit register expressions
  ($cpu:expr, A)  => { $cpu.regs.a() };
  ($cpu:expr, F)  => { $cpu.regs.f() };
  ($cpu:expr, B)  => { $cpu.regs.b() };
  ($cpu:expr, C)  => { $cpu.regs.c() };
  ($cpu:expr, D)  => { $cpu.regs.d() };
  ($cpu:expr, E)  => { $cpu.regs.e() };
  ($cpu:expr, H)  => { $cpu.regs.h() };
  ($cpu:expr, L)  => { $cpu.regs.l() };

  // 16-bit register expressions
  ($cpu:expr, AF) => { $cpu.regs.af() };
  ($cpu:expr, BC) => { $cpu.regs.bc() };
  ($cpu:expr, DE) => { $cpu.regs.de() };
  ($cpu:expr, HL) => { $cpu.regs.hl() };
  ($cpu:expr, PC) => { $cpu.regs.pc() };
  ($cpu:expr, SP) => { $cpu.regs.sp() };

  // Literal expressions
  ($cpu:expr, $val:expr) => {$val};
}

macro_rules! eval_flags {
  ($cpu:expr, INC BC : $c:expr) => { 0 };
  ($cpu:expr, INC DE : $c:expr) => { 0 };
  ($cpu:expr, INC HL : $c:expr) => { 0 };
  ($cpu:expr, INC SP : $c:expr) => { 0 };
  ($cpu:expr, INC IX : $c:expr) => { 0 };
  ($cpu:expr, INC IY : $c:expr) => { 0 };
  ($cpu:expr, INC $dst:tt :  $c:expr) => {
    let set = $c & 0b0010_1000;
    let reset = 0b1111_1101;
    eval!($cpu, F <- F | set);
    eval!($cpu, F <- F & reset);
  };
}

macro_rules! inst {
  ($cpu:expr, NOP) => {};
  ($cpu:expr, LD $dst:tt, $src:tt) => {
    eval!($cpu, $dst <- eval!($cpu, $src));
  };
  ($cpu:expr, INC $dst:tt) => {
    let c = eval!($cpu, $dst) + 1;
    eval!($cpu, $dst <- c);
    eval_flags!($cpu, INC $dst : c);
  };
}

#[cfg(test)]
mod test{
  use super::*;

  #[test]
  fn eval_regs8() {
    let mut cpu = CPU::new();

    eval!(cpu, A <- 0x40);
    eval!(cpu, F <- 0x41);
    eval!(cpu, B <- 0x42);
    eval!(cpu, C <- 0x43);
    eval!(cpu, D <- 0x44);
    eval!(cpu, E <- 0x45);
    eval!(cpu, H <- 0x46);
    eval!(cpu, L <- 0x47);

    assert_eq!(0x40, eval!(cpu, A));
    assert_eq!(0x41, eval!(cpu, F));
    assert_eq!(0x42, eval!(cpu, B));
    assert_eq!(0x43, eval!(cpu, C));
    assert_eq!(0x44, eval!(cpu, D));
    assert_eq!(0x45, eval!(cpu, E));
    assert_eq!(0x46, eval!(cpu, H));
    assert_eq!(0x47, eval!(cpu, L));
    assert_eq!(0x4041, eval!(cpu, AF));
    assert_eq!(0x4243, eval!(cpu, BC));
    assert_eq!(0x4445, eval!(cpu, DE));
    assert_eq!(0x4647, eval!(cpu, HL));
  }

  #[test]
  fn eval_regs16() {
    let mut cpu = CPU::new();

    eval!(cpu, AF <- 0x8081);
    eval!(cpu, BC <- 0x8283);
    eval!(cpu, DE <- 0x8485);
    eval!(cpu, HL <- 0x8687);
    eval!(cpu, PC <- 0x8889);
    eval!(cpu, SP <- 0x8A8B);

    assert_eq!(0x80, eval!(cpu, A));
    assert_eq!(0x81, eval!(cpu, F));
    assert_eq!(0x82, eval!(cpu, B));
    assert_eq!(0x83, eval!(cpu, C));
    assert_eq!(0x84, eval!(cpu, D));
    assert_eq!(0x85, eval!(cpu, E));
    assert_eq!(0x86, eval!(cpu, H));
    assert_eq!(0x87, eval!(cpu, L));

    assert_eq!(0x8889, eval!(cpu, PC));
    assert_eq!(0x8A8B, eval!(cpu, SP));
  }

  #[test]
  fn eval_regs_rhs() {
    let mut cpu = CPU::new();

    eval!(cpu, B <- 42);
    eval!(cpu, C <- B);

    assert_eq!(42, eval!(cpu, C));
  }

  #[test]
  fn eval_add() {
    let mut cpu = CPU::new();

    eval!(cpu, B <- 42);
    eval!(cpu, C <- 5);
    assert_eq!(47, eval!(cpu, B + C));

    eval!(cpu, A <- B + C);
    assert_eq!(47, eval!(cpu, A));

    eval!(cpu, A <- B + 12);
    assert_eq!(54, eval!(cpu, A));

    eval!(cpu, PC <- 0x1000);
    eval!(cpu, PC <- PC + 1);
    assert_eq!(0x1001, eval!(cpu, PC));

    eval!(cpu, HL <- 0x42);
    eval!(cpu, PC <- PC + HL + 1);
    assert_eq!(0x1044, eval!(cpu, PC));
  }

  #[test]
  fn inst_ld() {
    let mut cpu = CPU::new();

    inst!(cpu, LD HL, 0x1020);
    assert_eq!(0x1020, eval!(cpu, HL));
  }

  #[test]
  fn inst_inc() {
    let mut cpu = CPU::new();

    eval!(cpu, HL <- 0x1000);
    inst!(cpu, INC HL);
    assert_eq!(0x1001, eval!(cpu, HL));
    assert_eq!(0x00, eval!(cpu, F));

    eval!(cpu, A <- 42);
    inst!(cpu, INC A);
    assert_eq!(43, eval!(cpu, A));
    assert_eq!(0b0010_1000, eval!(cpu, F));
  }
}