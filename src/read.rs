use std::ops::Deref;

use collections::vec::Vec;

use error::Error;

pub trait Read<'de>: private::Sealed {
    fn input<'a>(&mut self, len: usize, scratch: &'a mut Vec<u8>) -> Result<Reference<'de, 'a>, Error>;
}

pub enum Reference<'de, 'a> {
    Borrowed(&'de [u8]),
    Copied(&'a [u8])
}

pub struct BorrowRead<'de, F: FnMut(usize) -> Result<&'de [u8], Error>> {
    thunk: F
}

pub struct CopyRead<F: FnMut(&mut [u8]) -> Result<(), Error>> {
    thunk: F
}

impl<'de, 'a> Deref for Reference<'de, 'a> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        match *self {
            Reference::Borrowed(data) => data,
            Reference::Copied(data) => data
        }
    }
}

impl<'de, F: FnMut(usize) -> Result<&'de [u8], Error>> BorrowRead<'de, F> {
    pub const fn new(thunk: F) -> BorrowRead<'de, F> {
        BorrowRead { thunk }
    }
}

impl<F: FnMut(&mut [u8]) -> Result<(), Error>> CopyRead<F> {
    pub const fn new(thunk: F) -> CopyRead<F> {
        CopyRead { thunk }
    }
}

impl<'de, F: FnMut(usize) -> Result<&'de [u8], Error>> private::Sealed for BorrowRead<'de, F> {}

impl<F: FnMut(&mut [u8]) -> Result<(), Error>> private::Sealed for CopyRead<F> {}

impl<'de, F: FnMut(usize) -> Result<&'de [u8], Error>> Read<'de> for BorrowRead<'de, F> {
    fn input<'a>(&mut self, len: usize, scratch: &'a mut Vec<u8>) -> Result<Reference<'de, 'a>, Error> {
        Ok(Reference::Borrowed((self.thunk)(len)?))
    }
}

impl<'de, F: FnMut(&mut [u8]) -> Result<(), Error>> Read<'de> for CopyRead<F> {
    fn input<'a>(&mut self, len: usize, scratch: &'a mut Vec<u8>) -> Result<Reference<'de, 'a>, Error> {
        scratch.resize(len, 0);
        (self.thunk)(scratch)?;
        Ok(Reference::Copied(scratch))
    }
}

mod private {
    pub trait Sealed {}
}