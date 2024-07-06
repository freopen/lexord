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

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
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

pub struct PrefixRead<R: Read> {
    pub prefix: Option<u8>,
    pub read: R,
}

impl<R: Read> From<R> for PrefixRead<R> {
    fn from(read: R) -> Self {
        Self { prefix: None, read }
    }
}

impl<R: Read> Read for PrefixRead<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if let Some(prefix) = self.prefix {
            buf[0] = prefix;
            self.prefix = None;
            Ok(1)
        } else {
            self.read.read(buf)
        }
    }
}

impl<R: Read> PrefixRead<R> {
    pub fn push(&mut self, prefix: u8) {
        assert!(
            self.prefix.is_none(),
            "Attempting to push a second prefix: {prefix:02X} -> PrefixRead({:02X})",
            self.prefix.unwrap()
        );
        self.prefix = Some(prefix);
    }
}

pub trait LexOrd: Sized + LexOrdSer {
    fn from_read<R: Read>(reader: &mut PrefixRead<R>) -> Result<Self>;
}
