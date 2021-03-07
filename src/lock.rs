//! This module contains an implementation of Pettersons Algorithm
//! This is needed to allow to synchronize the `usb-device` and `smoltcp` sides
//! of this crate with each other.
//!
//! We chose to implement a small lock on our own, to stay independent from the underlying hardware.
//! This lock only supports two accesors, and both have to be created in the same moment.

use core::{
   cell::UnsafeCell,
   ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub struct LockInner<'a, T> {
   data: &'a UnsafeCell<T>,
}

impl<'a, T> LockInner<'a, T> {
   pub fn new(data: &'a UnsafeCell<T>) -> (Self, Self) {
      (Self { data }, Self { data })
   }

   pub fn try_lock(&mut self) -> Option<Guard<'a, T>> {
      // TODO: Lock logic
      todo!()
   }
}

#[derive(Debug)]
pub struct Guard<'a, T> {
   lock: &'a mut LockInner<'a, T>,
}

impl<'a, T> Deref for Guard<'a, T> {
   type Target = T;
   fn deref(&self) -> &T {
      unsafe { &*self.lock.data.get() }
   }
}

impl<'a, T> DerefMut for Guard<'a, T> {
   fn deref_mut(&mut self) -> &mut T {
      unsafe { &mut *self.lock.data.get() }
   }
}

impl<'a, T> Drop for Guard<'a, T> {
   fn drop(&mut self) {
      // TODO: Unlock logic
   }
}
