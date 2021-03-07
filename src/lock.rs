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
pub struct Lock<'a, T> {
   inner: LockInner<T>,
   lock: LockHandle<'a, T>,
}

impl<'a, T> Lock<'a, T> {
   pub fn new(data: T) -> Self {
      let inner = LockInner::new(data);
      let ptr = &inner as *const LockInner<T> as *mut LockInner<T>;

      Self {
         inner,
         lock: LockHandle::new(ptr),
      }
   }

   pub fn get_handle(&self) -> LockHandle<'a, T> {
      self.lock.clone()
   }

   pub fn try_lock(&mut self) -> Option<Guard<'a, T>> {
      self.lock.try_lock()
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

   pub fn try_lock(&mut self) -> Option<Guard<'a, T>> {
      let inner: &mut LockInner<T> = unsafe { &mut *self.inner };

      todo!()
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
}

impl<T> LockInner<T> {
   fn new(data: T) -> Self {
      Self {
         lock: AtomicBool::new(false),
         data,
      }
   }
}

#[derive(Debug)]
pub struct Guard<'a, T> {
   lock: &'a mut LockHandle<'a, T>,
}

impl<'a, T> Deref for Guard<'a, T> {
   type Target = T;
   fn deref(&self) -> &T {
      &unsafe { &*self.lock.inner }.data
   }
}

impl<'a, T> DerefMut for Guard<'a, T> {
   fn deref_mut(&mut self) -> &mut T {
      &mut unsafe { &mut *self.lock.inner }.data
   }
}

impl<'a, T> Drop for Guard<'a, T> {
   fn drop(&mut self) {
      let inner: &mut LockInner<T> = unsafe { &mut *self.lock.inner };
      inner.lock.store(false, Ordering::SeqCst);
   }
}
