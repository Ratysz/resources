use downcast_rs::{impl_downcast, Downcast};
use fxhash::FxHashMap;
use parking_lot::RwLock;
use std::{any::TypeId, collections::hash_map as base, marker::PhantomData, ops::DerefMut};

use crate::{
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
/// Internally, this is a [`FxHashMap`] of [`TypeId`] to [`RwLock`].
/// The lock is used in a non-blocking way: attempting to access a resource in an invalid way will
/// return the appropriate error instead.
///
/// [`Resource`]: trait.Resource.html
/// [`FxHashMap`]: ../fxhash/type.FxHashMap.html
/// [`TypeId`]: https://doc.rust-lang.org/std/any/struct.TypeId.html
/// [`RwLock`]: ../lock_api/struct.RwLock.html
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
    /// Creates an empty container. Functionally identical to [`default`].
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
        Entry {
            base: self.resources.entry(TypeId::of::<T>()),
            phantom_data: PhantomData,
        }
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

/// A view into a specific [`Resource`]'s entry in a [`Resources`] container,
/// which may either be vacant or occupied.
///
/// This is returned by the [`entry`] method on [`Resources`].
///
/// [`Resource`]: trait.Resource.html
/// [`Resources`]: struct.Resources.html
/// [`entry`]: struct.Resources.html#method.entry
pub struct Entry<'a, T: Resource> {
    base: base::Entry<'a, TypeId, RwLock<Box<dyn Resource>>>,
    phantom_data: PhantomData<T>,
}

impl<'a, T: Resource> Entry<'a, T> {
    /// Ensures a resource is in the entry by inserting the given value if empty,
    /// and returns a mutable reference to the contained resource.
    pub fn or_insert(self, default: T) -> RefMut<'a, T> {
        self.or_insert_with(|| default)
    }

    /// Ensures a resource is in the entry by inserting the result of given function if empty,
    /// and returns a mutable reference to the contained resource.
    pub fn or_insert_with(self, default: impl FnOnce() -> T) -> RefMut<'a, T> {
        use base::Entry::*;
        RefMut::from_lock(match self.base {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.insert(RwLock::new(Box::new(default()))),
        })
        .expect("borrowing should always succeed here")
    }

    /// Provides in-place mutable access to an occupied entry before any potential inserts.
    pub fn and_modify(self, f: impl FnOnce(&mut T)) -> Self {
        use base::Entry::*;
        match self.base {
            Occupied(entry) => {
                f(RefMut::<'_, T>::from_lock(entry.get())
                    .expect("borrowing should always succeed here")
                    .deref_mut());
                Self {
                    base: Occupied(entry),
                    phantom_data: PhantomData,
                }
            }
            Vacant(entry) => Self {
                base: Vacant(entry),
                phantom_data: PhantomData,
            },
        }
    }
}

impl<'a, T: Resource + Default> Entry<'a, T> {
    /// Ensures a resource is in the entry by inserting it's default value if empty,
    /// and returns a mutable reference to the contained resource.
    pub fn or_default(self) -> RefMut<'a, T> {
        self.or_insert_with(T::default)
    }
}
