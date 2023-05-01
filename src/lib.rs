mod stdlib;
mod util;

use std::io::{Read, Write};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Parse error: {0}")]
    Parse(String),
}

pub type Result<T = ()> = std::result::Result<T, Error>;

pub trait LexOrd: Ord + Sized + LexOrdSer {
    fn from_read<R: Read>(reader: &mut R) -> Result<Self>;
}

pub enum ObjectType<T: ?Sized> {
    Default,
    CantStartWithZero,
    ZeroSized(fn() -> T),
}

pub trait LexOrdSer: Ord {
    const OBJECT_TYPE: ObjectType<Self> = ObjectType::Default;

    fn to_write<W: Write>(&self, writer: &mut W) -> Result;
}
