mod helpers;
mod stdlib;

use std::io::{Read, Write};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T = ()> = std::result::Result<T, Error>;

pub trait LexOrd: Ord + Sized {
    fn from_read<R: Read>(reader: &mut R) -> Result<Self>;
    fn to_write<W: Write>(&self, writer: &mut W) -> Result;
}
