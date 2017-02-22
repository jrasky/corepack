use std::fmt::Display;

use collections::String;

use alloc::boxed::Box;

use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum Reason {
    TooBig,
    ExtraItems,
    Unsized,
    BadValue,
    EndOfStream,
    BadType,
    BadLength,
    BadVariant,
    BadField,
    NoField,
    DupField,
    UTF8Error,
    Other,
}

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
            Reason::Unsized => "Unsized value",
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

    pub const fn new(reason: Reason, detail: String) -> Error {
        Error::chain(reason, detail, None)
    }

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
