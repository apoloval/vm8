use super::data::Data;
use super::ops::{Dest, Source};

pub enum Inst<T: Data> {
    Nop,
    Inc(Dest<T>),
    Load(Dest<T>, Source<T>),
}
