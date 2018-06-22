use super::data::Data;
use super::inst::{Context, Inst};
use super::ops::{Dest, Source};
use super::regs::Registers;

pub struct CPU {
    regs: Registers,
}

impl Context for CPU {
    fn regs(&self) -> &Registers { &self.regs }
    fn regs_mut(&mut self) -> &mut Registers { &mut self.regs }
}

impl CPU {
    pub fn exec<I: Inst>(&mut self, inst: I) {
        inst.exec(self)
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