use lock_api::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use std::ops::{Deref, DerefMut};

use crate::{lock::ResourcesRwLock, InvalidBorrow, Resource};

type Lock = RwLock<ResourcesRwLock, Box<dyn Resource>>;
type MappedReadGuard<'a, T> = MappedRwLockReadGuard<'a, ResourcesRwLock, T>;
type MappedWriteGuard<'a, T> = MappedRwLockWriteGuard<'a, ResourcesRwLock, T>;

pub struct Ref<'a, T: Resource> {
    read_guard: MappedReadGuard<'a, T>,
}

impl<'a, T: Resource> Ref<'a, T> {
    pub(crate) fn from_lock(lock: &'a Lock) -> Result<Self, InvalidBorrow> {
        lock.try_read()
            .map(|guard| Self {
                read_guard: RwLockReadGuard::map(guard, |resource| {
                    resource
                        .downcast_ref::<T>()
                        .unwrap_or_else(|| panic!("downcasting resources should always succeed"))
                }),
            })
            .ok_or_else(|| InvalidBorrow::Immutable)
    }
}

impl<'a, T: Resource> Deref for Ref<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.read_guard.deref()
    }
}

pub struct RefMut<'a, T: Resource> {
    write_guard: MappedWriteGuard<'a, T>,
}

impl<'a, T: Resource> RefMut<'a, T> {
    pub(crate) fn from_lock(lock: &'a Lock) -> Result<Self, InvalidBorrow> {
        lock.try_write()
            .map(|guard| Self {
                write_guard: RwLockWriteGuard::map(guard, |resource| {
                    resource
                        .downcast_mut::<T>()
                        .unwrap_or_else(|| panic!("downcasting resources should always succeed"))
                }),
            })
            .ok_or_else(|| InvalidBorrow::Mutable)
    }
}

impl<'a, T: Resource> Deref for RefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.write_guard.deref()
    }
}

impl<'a, T: Resource> DerefMut for RefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.write_guard.deref_mut()
    }
}
