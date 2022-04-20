use std::{
  cell::UnsafeCell,
  ops::{Deref, DerefMut},
  sync::atomic::{AtomicUsize, Ordering},
};

unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}

pub struct Mutex<T: ?Sized> {
  ticket: AtomicUsize,
  turn: UnsafeCell<usize>,
  value: UnsafeCell<T>,
}

impl<T> Mutex<T> {
  pub fn new(value: T) -> Self {
    Self {
      ticket: AtomicUsize::new(0),
      turn: UnsafeCell::new(0),
      value: UnsafeCell::new(value),
    }
  }
}

impl<T: ?Sized> Mutex<T> {
  pub fn lock(&'_ self) -> MutexGuard<'_, T> {
    let turn = self.ticket.fetch_add(1, Ordering::SeqCst);

    while turn != unsafe { *self.turn.get() } {
      // spin loop
    }

    MutexGuard { mutex: self }
  }

  fn unlock(&self) {
    unsafe {
      *self.turn.get() += 1;
    }
  }
}

pub struct MutexGuard<'a, T: ?Sized> {
  mutex: &'a Mutex<T>,
}

impl<'a, T: ?Sized> Deref for MutexGuard<'a, T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.mutex.value.get() }
  }
}

impl<'a, T: ?Sized> DerefMut for MutexGuard<'a, T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.mutex.value.get() }
  }
}

impl<'a, T: ?Sized> Drop for MutexGuard<'a, T> {
  fn drop(&mut self) {
    self.mutex.unlock();
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use super::*;

  #[test]
  fn smoke() {
    let mutex = Arc::new(Mutex::new(0));

    let mut handles = Vec::new();

    for _ in 0..2 {
      let mutex = Arc::clone(&mutex);

      handles.push(std::thread::spawn(move || {
        for _ in 0..1000 {
          let mut guard = mutex.lock();
          *guard += 1;
        }
      }));
    }

    for handle in handles.into_iter() {
      handle.join().expect("error joining thread");
    }

    assert_eq!(2000, *mutex.lock());
  }
}
