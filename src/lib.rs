//! [![Latest Version]][crates.io] [![Documentation]][docs.rs] [![Dependencies]][deps.rs] [![License]][license link]
//!
//! This crate provides the [`Resources`] struct:
//! a container that stores at most one value of each specific type,
//! and allows safely and concurrently accessing any of them with interior mutability,
//! without violating borrow rules.
//!
//! It's intended to be used as an implementation of storage for data that is not
//! associated with any specific entity in an ECS (entity component system),
//! but still needs concurrent access by systems.
//!
//! ```rust
//!use resources::*;
//!
//!struct SomeNumber(usize);
//!
//!struct SomeString(&'static str);
//!
//!fn main() {
//!    let mut resources = Resources::new();
//!    resources.insert(SomeNumber(4));
//!    resources.insert(SomeString("Hello!"));
//!    let resources = resources;  // This shadows the mutable binding with an immutable one.
//!
//!    {
//!        let mut some_number = resources.get_mut::<SomeNumber>().unwrap();
//!        let mut some_string = resources.get_mut::<SomeString>().unwrap();
//!
//!        // Immutably borrowing a resource that's already borrowed mutably is not allowed.
//!        assert!(resources.get::<SomeNumber>().is_err());
//!
//!        some_number.0 = 2;
//!        some_string.0 = "Bye!";
//!    }
//!
//!    // The mutable borrows now went out of scope, so it's okay to borrow again however needed.
//!    assert_eq!(resources.get::<SomeNumber>().unwrap().0, 2);
//!
//!    // Multiple immutable borrows are okay.
//!    let some_string1 = resources.get::<SomeString>().unwrap();
//!    let some_string2 = resources.get::<SomeString>().unwrap();
//!    assert_eq!(some_string1.0, some_string2.0);
//!}
//!```
//!
//! [`Resources`]: struct.Resources.html
//! [Latest Version]: https://img.shields.io/crates/v/resources.svg
//! [crates.io]: https://crates.io/crates/resources
//! [Documentation]: https://docs.rs/resources/badge.svg
//! [docs.rs]: https://docs.rs/resources
//! [Dependencies]: https://deps.rs/repo/github/Ratysz/resources/status.svg
//! [deps.rs]: https://deps.rs/repo/github/Ratysz/resources
//! [License]: https://img.shields.io/crates/l/resources.svg
//! [license link]: https://github.com/Ratysz/resources/LICENSE.md

#![warn(missing_docs)]

mod error;
mod lock;
mod map;
mod refs;

pub use error::{CantGetResource, InvalidBorrow, NoSuchResource};
pub use map::{Entry, Resource, Resources};
pub use refs::{Ref, RefMut};
