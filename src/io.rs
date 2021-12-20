use std::boxed::Box;
use std::marker::PhantomData;

use crate::bus::{Address, Bus, Data};

pub struct Linear<A: Address, D: Data> {
    address: PhantomData<A>,
    ports: Vec<PortEntry<D>>,
}

impl<A: Address, D: Data> Linear<A, D> {
    pub fn new() -> Self {
        let address = PhantomData;
        let size: usize = A::max_value().into() - A::min_value().into() + 1;
        let mut ports: Vec<PortEntry<D>> = Vec::with_capacity(size);
        for _ in 0..size {
            ports.push(None);
        }
        Self { address, ports }
    }

    pub fn bind(&mut self, a: A, port: Box<dyn Port<D>>) {
        self.ports[a.into()] = Some(port);
    }
}

impl<A: Address, D: Data> Bus<A, D> for Linear<A, D> {
    fn read_from(&mut self, addr: A) -> D {
        match self.ports[addr.into()] {
            Some(ref mut port) => port.read(),
            None => D::default(),
        }
    }

    fn write_to(&mut self, addr: A, val: D) {
        if let Some(ref mut port) = self.ports[addr.into()] {
            port.write(val);
        }
    }
}

pub trait Port<D: Data> {
    fn read(&mut self) -> D;
    fn write(&mut self, value: D);
}

pub struct Register<D: Data> {
    value: D,
}

impl<D: Data> Register<D> {
    pub fn new() -> Self {
        Self { value: D::default() }
    }
}

impl<D: Data> Port<D> for Register<D> {
    fn read(&mut self) -> D { self.value }
    fn write(&mut self, value: D) { self.value = value; }
}

type PortEntry<D> = Option<Box<dyn Port<D>>>;