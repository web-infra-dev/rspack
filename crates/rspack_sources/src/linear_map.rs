pub struct LinearMap<V: Default> {
  inner: Vec<V>,
}

impl<V: Default> LinearMap<V> {
  pub fn new() -> Self {
    Self {
      inner: Default::default(),
    }
  }

  #[inline]
  pub fn get(&self, key: &u32) -> Option<&V> {
    self.inner.get(*key as usize)
  }

  #[inline]
  pub fn get_mut(&mut self, key: &u32) -> Option<&mut V> {
    self.inner.get_mut(*key as usize)
  }

  #[inline]
  pub fn insert(&mut self, key: u32, value: V) {
    let key = key as usize;
    if key >= self.inner.len() {
      self.inner.resize_with(key + 1, Default::default);
    }
    self.inner[key] = value;
  }

  #[inline]
  pub fn clear(&mut self) {
    self.inner.clear()
  }
}

impl<V: Default> Default for LinearMap<V> {
  fn default() -> Self {
    Self::new()
  }
}
