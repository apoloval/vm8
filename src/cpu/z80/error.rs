use std::result::{Result as StdResult};

pub enum Error {}

pub type Result<T> = StdResult<T, Error>;
