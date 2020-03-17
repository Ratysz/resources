use downcast_rs::{impl_downcast, Downcast};
use fxhash::FxHashMap;
use parking_lot::RwLock;
use std::any::TypeId;

use crate::{
    entry::Entry,
    error::{CantGetResource, NoSuchResource},
    refs::{Ref, RefMut},
};

/// Types that can be stored in [`Resources`], automatically implemented for all applicable.
///
/// [`Resources`]: struct.Resources.html
pub trait Resource: Downcast + Send + Sync + 'static {}

impl<T> Resource for T where T: Send + Sync + 'static {}

impl_downcast!(Resource);

/// A [`Resource`] container, for storing at most one resource of each specific type.
///
/// Internally, this is a [`FxHashMap`] of [`TypeId`] to [`RwLock`]. None of the methods are
/// blocking, however: accessing a resource in a way that would break borrow rules will
/// return the [`InvalidBorrow`] error instead.
///
/// [`Resource`]: trait.Resource.html
/// [`FxHashMap`]: ../fxhash/type.FxHashMap.html
/// [`TypeId`]: https://doc.rust-lang.org/std/any/struct.TypeId.html
/// [`RwLock`]: ../parking_lot/type.RwLock.html
/// [`InvalidBorrow`]: enum.InvalidBorrow.html
#[derive(Default)]
pub struct Resources {
    resources: FxHashMap<TypeId, RwLock<Box<dyn Resource>>>,
}

fn downcast_resource<T: Resource>(resource: Box<dyn Resource>) -> T {
    *resource
        .downcast::<T>()
        .unwrap_or_else(|_| panic!("downcasting resources should always succeed"))
}

impl Resources {
    /// Creates an empty container. Functionally identical to [`::default()`].
    ///
    /// [`default`]: #method.default
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if a resource of type `T` exists in the container.
    pub fn contains<T: Resource>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<T>())
    }

    /// Inserts the given resource of type `T` into the container.
    ///
    /// If a resource of this type was already present,
    /// it will be updated, and the original returned.
    pub fn insert<T: Resource>(&mut self, resource: T) -> Option<T> {
        self.resources
            .insert(TypeId::of::<T>(), RwLock::new(Box::new(resource)))
            .map(|resource| downcast_resource(resource.into_inner()))
    }

    /// Removes the resource of type `T` from the container.
    ///
    /// If a resource of this type was present in the container, it will be returned.
    pub fn remove<T: Resource>(&mut self) -> Option<T> {
        self.resources
            .remove(&TypeId::of::<T>())
            .map(|resource| downcast_resource(resource.into_inner()))
    }

    /// Gets the type `T`'s corresponding entry for in-place manipulation.
    pub fn entry<T: Resource>(&mut self) -> Entry<T> {
        Entry::from_hash_map_entry(self.resources.entry(TypeId::of::<T>()))
    }

    /// Returns a reference to the stored resource of type `T`.
    ///
    /// If such a resource is currently accessed mutably elsewhere,
    /// or is not present in the container, returns the appropriate error.
    pub fn get<T: Resource>(&self) -> Result<Ref<T>, CantGetResource> {
        self.resources
            .get(&TypeId::of::<T>())
            .ok_or_else(|| NoSuchResource.into())
            .and_then(|lock| Ref::from_lock(lock).map_err(|error| error.into()))
    }

    /// Returns a mutable reference to the stored resource of type `T`.
    ///
    /// If such a resource is currently accessed immutably or mutably elsewhere,
    /// or is not present in the container, returns the appropriate error.
    pub fn get_mut<T: Resource>(&self) -> Result<RefMut<T>, CantGetResource> {
        self.resources
            .get(&TypeId::of::<T>())
            .ok_or_else(|| NoSuchResource.into())
            .and_then(|lock| RefMut::from_lock(lock).map_err(|error| error.into()))
    }
}
