use super::data::Data;
use super::inst::Inst;
use super::ops::{Dest, Source};
use super::regs::Registers;

pub struct CPU {
    regs: Registers,
}

impl CPU {
    pub fn exec<T: Data>(&mut self, inst: &Inst<T>) {
        match inst {
            Inst::Inc(dst) => self.exec_inc(dst),
            Inst::Nop => self.exec_nop(),
            Inst::Load(dst, src) => self.exec_load(dst, src),
        }
    }

    fn exec_inc<T: Data>(&mut self, dst: &Dest<T>) {
        let val = self.read_dst(dst);
        self.write_dest(dst, T::inc(val));
    }

    fn exec_nop(&mut self) {}

    fn exec_load<T: Data>(&mut self, dst: &Dest<T>, src: &Source<T>) {
        let val = self.read_src(src);
        self.write_dest(dst, val);
    }

    fn read_src<T: Data>(&self, op: &Source<T>) -> T::Value {
        match op {
            Source::Literal(v) => *v,
            Source::Register(r) => self.regs.read::<T>(*r),
        }
    }

    fn read_dst<T: Data>(&self, op: &Dest<T>) -> T::Value {
        match op {
            Dest::Register(r) => self.regs.read::<T>(*r),
        }
    }

    fn write_dest<T: Data>(&mut self, op: &Dest<T>, val: T::Value) {
        match op {
            Dest::Register(r) => self.regs.write::<T>(*r, val),
        }
    }
}