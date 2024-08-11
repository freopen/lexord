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

struct SeqWriter<'a, W: Write> {
    writer: &'a mut W,
    start: bool,
}

impl<'a, W: Write> SeqWriter<'a, W> {
    fn new(writer: &'a mut W) -> Self {
        SeqWriter {
            writer,
            start: true,
        }
    }
}

impl<'a, W: Write> Write for SeqWriter<'a, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        if self.start && buf[0] <= 0x01 {
            self.writer.write_all(&[0x01])?;
        }
        self.start = false;
        self.writer.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl<'a, W: Write> Drop for SeqWriter<'a, W> {
    fn drop(&mut self) {
        if self.start {
            self.writer.write_all(&[0x01]).unwrap();
        }
    }
}

pub trait LexOrdSer: PartialOrd {
    fn to_write(&self, writer: &mut impl Write) -> Result;
    fn to_write_seq(&self, writer: &mut impl Write) -> Result {
        self.to_write(&mut SeqWriter::new(writer))
    }
}

impl<T: LexOrdSer> LexOrdSer for &T {
    fn to_write(&self, writer: &mut impl Write) -> Result {
        T::to_write(self, writer)
    }
    fn to_write_seq(&self, writer: &mut impl Write) -> Result {
        T::to_write_seq(self, writer)
    }
}

pub trait LexOrd: Sized + LexOrdSer {
    fn from_read(reader: &mut impl Read) -> Result<Self>;
    fn from_read_seq(first: u8, reader: &mut impl Read) -> Result<Self> {
        if first == 0x01 {
            Self::from_read(reader)
        } else {
            Self::from_read(&mut [first].chain(reader))
        }
    }
}
