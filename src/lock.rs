use lock_api::{GuardSend, RawRwLock};
use std::sync::atomic::{AtomicIsize, Ordering};

pub(crate) struct ResourcesRwLock {
    counter: AtomicIsize,
}

enum LockState {
    Free,
    Exclusive,
    Shared,
}

use LockState::*;

impl ResourcesRwLock {
    fn state(&self) -> LockState {
        match self.counter.load(Ordering::SeqCst) {
            0 => Free,
            -1 => Exclusive,
            _ => Shared,
        }
    }

    unsafe fn lock_shared_unchecked(&self) {
        self.counter.fetch_add(1, Ordering::SeqCst);
    }

    unsafe fn unlock_shared_unchecked(&self) {
        self.counter.fetch_sub(1, Ordering::SeqCst);
    }

    unsafe fn lock_exclusive_unchecked(&self) {
        self.counter.store(-1, Ordering::SeqCst);
    }

    unsafe fn unlock_exclusive_unchecked(&self) {
        self.counter.store(0, Ordering::SeqCst);
    }
}

unsafe impl RawRwLock for ResourcesRwLock {
    const INIT: Self = ResourcesRwLock {
        counter: AtomicIsize::new(0),
    };

    type GuardMarker = GuardSend;

    fn lock_shared(&self) {
        if !self.try_lock_shared() {
            match self.state() {
                Free => unreachable!(),
                Exclusive => panic!("non-exclusive lock while exclusively locked"),
                Shared => unreachable!(),
            }
        }
    }

    fn try_lock_shared(&self) -> bool {
        let can_lock = match self.state() {
            Free => true,
            Exclusive => false,
            Shared => true,
        };
        if can_lock {
            unsafe {
                self.lock_shared_unchecked();
            }
        }
        can_lock
    }

    fn unlock_shared(&self) {
        match self.state() {
            Free => panic!("non-exclusive unlock while open"),
            Exclusive => panic!("non-exclusive unlock while exclusively locked"),
            Shared => unsafe { self.unlock_shared_unchecked() },
        }
    }

    fn lock_exclusive(&self) {
        if !self.try_lock_exclusive() {
            match self.state() {
                Exclusive => panic!("exclusive lock while exclusively locked"),
                Shared => panic!("exclusive lock while non-exclusively locked"),
                _ => unreachable!(),
            }
        }
    }

    fn try_lock_exclusive(&self) -> bool {
        let can_lock = match self.state() {
            Free => true,
            Exclusive => false,
            Shared => false,
        };
        if can_lock {
            unsafe { self.lock_exclusive_unchecked() }
        }
        can_lock
    }

    fn unlock_exclusive(&self) {
        match self.state() {
            Free => panic!("exclusive unlock while open"),
            Exclusive => unsafe { self.unlock_exclusive_unchecked() },
            Shared => panic!("exclusive unlock while non-exclusively locked"),
        }
    }
}
