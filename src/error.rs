use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NoSuchResource;

impl Display for NoSuchResource {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.pad("no such resource")
    }
}

impl Error for NoSuchResource {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum InvalidBorrow {
    Mutable,
    Immutable,
}

impl Display for InvalidBorrow {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.pad(match self {
            InvalidBorrow::Mutable => "cannot borrow mutably",
            InvalidBorrow::Immutable => "cannot borrow immutably",
        })
    }
}

impl Error for InvalidBorrow {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CantGetResource {
    InvalidBorrow(InvalidBorrow),
    NoSuchResource(NoSuchResource),
}

impl Display for CantGetResource {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use CantGetResource::*;
        match self {
            InvalidBorrow(error) => error.fmt(f),
            NoSuchResource(error) => error.fmt(f),
        }
    }
}

impl Error for CantGetResource {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use CantGetResource::*;
        match self {
            InvalidBorrow(error) => Some(error),
            NoSuchResource(error) => Some(error),
        }
    }
}

impl From<NoSuchResource> for CantGetResource {
    fn from(error: NoSuchResource) -> Self {
        CantGetResource::NoSuchResource(error)
    }
}

impl From<InvalidBorrow> for CantGetResource {
    fn from(error: InvalidBorrow) -> Self {
        CantGetResource::InvalidBorrow(error)
    }
}
