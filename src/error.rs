//! Error types for corepack.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt::Display;

use collections::String;

use alloc::boxed::Box;

use std::fmt;

/// Reasons that parsing or encoding might fail in corepack.
#[derive(Debug, Clone, Copy)]
pub enum Reason {
    /// Container or sequence was too big to serialize.
    TooBig,

    /// Extra items remained after deserializing a sequence.
    ExtraItems,

    /// Invalid value encountered.
    BadValue,

    /// Reached end of a stream.
    EndOfStream,

    /// Invalid type encountered.
    BadType,

    /// Invalid length encountered.
    BadLength,

    /// Encountered an unknown enum variant.
    BadVariant,

    /// Unknown field included in struct.
    BadField,

    /// Missing field from struct.
    NoField,

    /// Duplicate field found in struct.
    DupField,

    /// Error decoding UTF8 string.
    UTF8Error,

    /// Some other error that does not fit into the above.
    Other,
}

/// Error struct for corepack errors.
#[derive(Debug)]
pub struct Error {
    reason: Reason,
    detail: String,
    #[cfg(not(feature = "std"))]
    cause: Option<Box<::serde::error::Error>>,
    #[cfg(feature = "std")]
    cause: Option<Box<::std::error::Error>>,
}

impl Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let name = match self.reason {
            Reason::TooBig => "Overflowing value",
            Reason::ExtraItems => "More items that expected",
            Reason::BadValue => "Invalid value",
            Reason::EndOfStream => "End of stream",
            Reason::BadType => "Invalid type",
            Reason::BadLength => "Invalid length",
            Reason::BadVariant => "Unknown variant",
            Reason::BadField => "Unknown field",
            Reason::NoField => "Missing field",
            Reason::DupField => "Duplicate field",
            Reason::UTF8Error => "UTF-8 encoding error",
            Reason::Other => "Other error",
        };

        if !self.detail.is_empty() {
            write!(fmt, "{}: {}", name, self.detail)
        } else {
            write!(fmt, "{}", name)
        }
    }
}

impl Error {
    /// Wrap an error in a new error, for context.
    #[cfg(not(feature = "std"))]
    pub const fn chain(reason: Reason,
                       detail: String,
                       cause: Option<Box<::serde::error::Error>>)
                       -> Error {
        Error {
            reason: reason,
            detail: detail,
            cause: cause,
        }
    }

    /// Wrap an error in a new error, for context.
    #[cfg(feature = "std")]
    pub const fn chain(reason: Reason,
                       detail: String,
                       cause: Option<Box<::std::error::Error>>)
                       -> Error {
        Error {
            reason: reason,
            detail: detail,
            cause: cause,
        }
    }

    /// Create a new error without chaining a cause.
    pub const fn new(reason: Reason, detail: String) -> Error {
        Error::chain(reason, detail, None)
    }


    /// Create a new error from just a reason.
    pub fn simple(reason: Reason) -> Error {
        Error::new(reason, String::new())
    }
}

#[cfg(not(feature = "std"))]
impl ::serde::error::Error for Error {
    fn description(&self) -> &str {
        "Corepack error"
    }

    fn cause(&self) -> Option<&::serde::error::Error> {
        if let Some(ref e) = self.cause {
            Some(e.as_ref())
        } else {
            None
        }
    }
}

#[cfg(feature = "std")]
impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        "Corepack error"
    }

    fn cause(&self) -> Option<&::std::error::Error> {
        if let Some(ref e) = self.cause {
            Some(e.as_ref())
        } else {
            None
        }
    }
}

impl ::serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Error {
        Error::new(Reason::Other, format!("{}", msg))
    }
}

impl ::serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Error {
        ::serde::ser::Error::custom(msg)
    }
}
