use crate::ModuleIdentifier;

#[derive(Debug, Default)]
pub struct Mutations {
  inner: Vec<Mutation>,
}

#[derive(Debug)]
pub enum Mutation {
  ModuleRevoke { module: ModuleIdentifier },
  ModuleSetAsync { module: ModuleIdentifier },
}

impl Mutations {
  pub fn add(&mut self, mutation: Mutation) {
    self.inner.push(mutation);
  }
}

impl Mutations {
  pub fn iter(&self) -> std::slice::Iter<Mutation> {
    self.inner.iter()
  }
}

pub struct IntoIter {
  inner: std::vec::IntoIter<Mutation>,
}

impl Iterator for IntoIter {
  type Item = Mutation;

  fn next(&mut self) -> Option<Self::Item> {
    self.inner.next()
  }
}

impl IntoIterator for Mutations {
  type Item = Mutation;
  type IntoIter = IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    IntoIter {
      inner: self.inner.into_iter(),
    }
  }
}

impl Extend<Mutation> for Mutations {
  fn extend<T: IntoIterator<Item = Mutation>>(&mut self, iter: T) {
    self.inner.extend(iter);
  }
}
