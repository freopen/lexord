mod stdlib;
pub mod util;

pub use lexord_derive::LexOrd;

use std::{
    convert::Infallible,
    io::{Read, Write},
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Int parse error: {0}")]
    FromInt(#[from] std::num::TryFromIntError),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

pub type Result<T = ()> = std::result::Result<T, Error>;

pub enum ObjectType {
    Default,
    CantStartWithZero,
    ZeroSized,
}

impl ObjectType {
    pub const fn sequence_type(seq: &[ObjectType]) -> ObjectType {
        match seq {
            [] => ObjectType::ZeroSized,
            [ObjectType::CantStartWithZero, ..] => ObjectType::CantStartWithZero,
            [ObjectType::Default, ..] => ObjectType::Default,
            [ObjectType::ZeroSized, rest @ ..] => ObjectType::sequence_type(rest),
        }
    }
}

pub trait LexOrdSer: PartialOrd {
    fn object_type() -> ObjectType {
        ObjectType::Default
    }

    fn to_write<W: Write>(&self, writer: &mut W) -> Result;
}

impl<T: LexOrdSer> LexOrdSer for &T {
    fn object_type() -> ObjectType {
        T::object_type()
    }

    fn to_write<W: Write>(&self, writer: &mut W) -> Result {
        T::to_write(self, writer)
    }
}

pub trait LexOrd: Sized + LexOrdSer {
    fn from_read<R: Read>(reader: &mut R) -> Result<Self>;
}
