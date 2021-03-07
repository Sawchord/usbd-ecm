//! This module contains very basic lock implementation based on atomic bool.
//!
//! This is needed to allow to synchronize the `usb-device` and `smoltcp` sides
//! of this crate with each other.
//!
//! We chose to implement a small lock on our own, to stay independent from the underlying hardware.
//! This lock only supports two accesors, and both have to be created in the same moment.

use core::{
   marker::PhantomData,
   ops::{Deref, DerefMut},
   sync::atomic::{AtomicBool, Ordering},
};

#[derive(Debug)]
pub struct Lock<T>(LockInner<T>);

impl<T> Lock<T> {
   pub fn new(data: T) -> Self {
      Self(LockInner::new(data))
   }

   pub fn get_handle(&self) -> LockHandle<T> {
      LockHandle::new(self.0.as_ptr())
   }

   pub fn try_lock(&self) -> Option<Guard<T>> {
      self.0.try_lock()
   }
}

#[derive(Debug)]
pub struct LockHandle<'a, T> {
   lt: &'a PhantomData<()>,
   inner: *mut LockInner<T>,
}

impl<'a, T> LockHandle<'a, T> {
   pub fn new(inner: *mut LockInner<T>) -> Self {
      Self {
         lt: &PhantomData,
         inner,
      }
   }

   pub fn try_lock(&'a self) -> Option<Guard<'a, T>> {
      let inner: &mut LockInner<T> = unsafe { &mut *self.inner };
      inner.try_lock()
   }
}

impl<'a, T> Clone for LockHandle<'a, T> {
   fn clone(&self) -> Self {
      Self {
         lt: &PhantomData,
         inner: self.inner,
      }
   }
}

#[derive(Debug)]
pub struct LockInner<T> {
   lock: AtomicBool,
   data: T,
   // This is to make LockInner `!Send` and `!Sync`.
   unsend: *const PhantomData<()>,
}

impl<T> LockInner<T> {
   fn new(data: T) -> Self {
      Self {
         lock: AtomicBool::new(false),
         data,
         unsend: &PhantomData as *const PhantomData<()>,
      }
   }
}

impl<T> LockInner<T> {
   fn try_lock(&self) -> Option<Guard<T>> {
      let was_locked = self
         .lock
         .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst);

      match was_locked {
         Ok(false) => Some(Guard(LockHandle::new(self.as_ptr()))),
         _ => None,
      }
   }

   fn as_ptr(&self) -> *mut Self {
      self as *const Self as *mut Self
   }
}

#[derive(Debug)]
pub struct Guard<'a, T>(LockHandle<'a, T>);

impl<'a, T> Deref for Guard<'a, T> {
   type Target = T;
   fn deref(&self) -> &T {
      &unsafe { &*self.0.inner }.data
   }
}

impl<T> DerefMut for Guard<'_, T> {
   fn deref_mut(&mut self) -> &mut T {
      &mut unsafe { &mut *self.0.inner }.data
   }
}

impl<T> Drop for Guard<'_, T> {
   fn drop(&mut self) {
      let inner: &mut LockInner<T> = unsafe { &mut *self.0.inner };
      inner.lock.store(false, Ordering::SeqCst);
   }
}
