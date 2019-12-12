use downcast_rs::{impl_downcast, Downcast};
use fxhash::FxHashMap;
use lock_api::RwLock;
use std::{any::TypeId, collections::hash_map as base, marker::PhantomData, ops::DerefMut};

use crate::{
    error::{CantGetResource, NoSuchResource},
    lock::ResourcesRwLock,
    refs::{Ref, RefMut},
};

type Lock = RwLock<ResourcesRwLock, Box<dyn Resource>>;

pub trait Resource: Downcast + Send + Sync + 'static {}

impl<T> Resource for T where T: Send + Sync + 'static {}

impl_downcast!(Resource);

#[derive(Default)]
pub struct Resources {
    resources: FxHashMap<TypeId, Lock>,
}

fn downcast_resource<T: Resource>(resource: Box<dyn Resource>) -> T {
    *resource
        .downcast::<T>()
        .unwrap_or_else(|_| panic!("downcasting resources should always succeed"))
}

impl Resources {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains<T: Resource>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<T>())
    }

    pub fn insert<T: Resource>(&mut self, resource: T) -> Option<T> {
        self.resources
            .insert(TypeId::of::<T>(), RwLock::new(Box::new(resource)))
            .map(|resource| downcast_resource(resource.into_inner()))
    }

    pub fn remove<T: Resource>(&mut self) -> Result<T, NoSuchResource> {
        self.resources
            .remove(&TypeId::of::<T>())
            .map(|resource| downcast_resource(resource.into_inner()))
            .ok_or_else(|| NoSuchResource)
    }

    pub fn entry<T: Resource>(&mut self) -> Entry<T> {
        Entry {
            base: self.resources.entry(TypeId::of::<T>()),
            phantom_data: PhantomData,
        }
    }

    pub fn get<T: Resource>(&self) -> Result<Ref<T>, CantGetResource> {
        self.resources
            .get(&TypeId::of::<T>())
            .ok_or_else(|| NoSuchResource.into())
            .and_then(|lock| Ref::from_lock(lock).map_err(|error| error.into()))
    }

    pub fn get_mut<T: Resource>(&self) -> Result<RefMut<T>, CantGetResource> {
        self.resources
            .get(&TypeId::of::<T>())
            .ok_or_else(|| NoSuchResource.into())
            .and_then(|lock| RefMut::from_lock(lock).map_err(|error| error.into()))
    }
}

pub struct Entry<'a, T: Resource> {
    base: base::Entry<'a, TypeId, RwLock<ResourcesRwLock, Box<dyn Resource>>>,
    phantom_data: PhantomData<T>,
}

impl<'a, T: Resource> Entry<'a, T> {
    pub fn or_insert(self, default: T) -> RefMut<'a, T> {
        self.or_insert_with(|| default)
    }

    pub fn or_insert_with(self, default: impl FnOnce() -> T) -> RefMut<'a, T> {
        use base::Entry::*;
        RefMut::from_lock(match self.base {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.insert(RwLock::new(Box::new(default()))),
        })
        .expect("borrowing should always succeed here")
    }

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
    pub fn or_default(self) -> RefMut<'a, T> {
        self.or_insert_with(T::default)
    }
}
