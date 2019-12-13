#![warn(missing_docs)]

mod error;
mod lock;
mod map;
mod refs;

pub use error::{CantGetResource, InvalidBorrow, NoSuchResource};
pub use map::{Entry, Resource, Resources};
pub use refs::{Ref, RefMut};
