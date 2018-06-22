use super::data::Data;

pub enum Source<T: Data> {
    Literal(T::Value),
    Register(T::Reg),
}

pub enum Dest<T: Data> {
    Register(T::Reg),
}
