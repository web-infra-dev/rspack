use std::{cell::RefCell, collections::BTreeMap};

// Vector pooling minimum capacity threshold
// Recommended threshold: 64
// Reasons:
// 1. Memory consideration: 64 * 8 bytes = 512 bytes, a reasonable memory block size
// 2. Allocation cost: Allocations smaller than 512 bytes are usually fast, pooling benefits are limited
// 3. Cache friendly: 512 bytes can typically utilize CPU cache well
// 4. Empirical value: 64 is a proven balance point in real projects
const MIN_POOL_CAPACITY: usize = 64;

/// A memory pool for reusing `T` allocations to reduce memory allocation overhead.
#[derive(Default, Debug)]
pub struct ObjectPool {
  objects: RefCell<BTreeMap<usize, Vec<Vec<usize>>>>,
}

impl ObjectPool {
  /// Retrieves a reusable `T` from the pool with at least the requested capacity.
  pub fn pull<'a>(&'a self, requested_capacity: usize) -> Pooled<'a> {
    if requested_capacity < MIN_POOL_CAPACITY || self.objects.borrow().is_empty() {
      return Pooled::new(self, Vec::with_capacity(requested_capacity));
    }
    let mut objects = self.objects.borrow_mut();
    if let Some((_, bucket)) = objects.range_mut(requested_capacity..).next() {
      if let Some(mut object) = bucket.pop() {
        object.clear();
        return Pooled::new(self, object);
      }
    }
    Pooled::new(self, Vec::with_capacity(requested_capacity))
  }

  /// Returns a `T` to the pool for future reuse.
  fn return_to_pool(&self, object: Vec<usize>) {
    if object.capacity() < MIN_POOL_CAPACITY {
      return;
    }
    let mut objects = self.objects.borrow_mut();
    let cap = object.capacity();
    let bucket = objects.entry(cap).or_default();
    bucket.push(object);
  }
}

/// A smart pointer that holds a pooled object and automatically returns it to the pool when dropped.
///
/// `Pooled<T>` implements RAII (Resource Acquisition Is Initialization) pattern to manage
/// pooled objects lifecycle. When the `Pooled` instance is dropped, the contained object
/// is automatically returned to its associated pool for future reuse.
#[derive(Debug)]
pub struct Pooled<'object_pool> {
  object: Option<Vec<usize>>,
  pool: &'object_pool ObjectPool,
}

impl<'object_pool> Pooled<'object_pool> {
  fn new(pool: &'object_pool ObjectPool, object: Vec<usize>) -> Self {
    Pooled {
      object: Some(object),
      pool,
    }
  }

  pub fn as_mut(&mut self) -> &mut Vec<usize> {
    self.object.as_mut().unwrap()
  }

  pub fn as_ref(&self) -> &Vec<usize> {
    self.object.as_ref().unwrap()
  }
}

impl Drop for Pooled<'_> {
  fn drop(&mut self) {
    if let Some(object) = self.object.take() {
      self.pool.return_to_pool(object);
    }
  }
}

impl std::ops::Deref for Pooled<'_> {
  type Target = Vec<usize>;

  fn deref(&self) -> &Self::Target {
    self.as_ref()
  }
}

impl std::ops::DerefMut for Pooled<'_> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.as_mut()
  }
}
