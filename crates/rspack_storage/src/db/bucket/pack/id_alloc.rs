use itertools::Itertools;

use super::{Error, PackId, Result};

#[derive(Debug)]
pub struct PackIdAlloc {
  next: PackId,
  reuse_pool: Vec<PackId>,
}

impl Default for PackIdAlloc {
  fn default() -> Self {
    // pack id begin at 1, hot pack id is 0
    Self {
      next: PackId::new(1),
      reuse_pool: vec![],
    }
  }
}

impl PackIdAlloc {
  pub const HOT_PACK_ID: PackId = PackId::new(0);

  pub fn try_from_string(s: &str) -> Result<Self> {
    let mut inner = vec![];
    for item in s.split(" ") {
      inner.push(item.try_into()?);
    }
    // last is next
    let Some(next) = inner.pop() else {
      return Err(Error::InvalidFormat("pack id alloc not allow empty".into()));
    };

    Ok(Self {
      next,
      reuse_pool: inner,
    })
  }

  pub fn to_string(&self) -> String {
    self
      .reuse_pool
      .iter()
      // add next to last
      .chain(std::iter::once(&self.next))
      .map(|item| item.to_string())
      .join(" ")
  }

  pub fn next_id(&mut self) -> PackId {
    if let Some(id) = self.reuse_pool.pop() {
      return id;
    }
    let next = self.next;
    self.next = PackId::new(next.inner() + 1);
    next
  }

  pub fn add_id(&mut self, id: PackId) {
    if id != Self::HOT_PACK_ID && id < self.next {
      self.reuse_pool.push(id);
    }
  }
}

#[cfg(test)]
mod test {
  use super::{PackId, PackIdAlloc};

  #[test]
  fn skip_hot_pack_id() {
    let mut alloc = PackIdAlloc::default();
    assert!(alloc.next_id().inner() > PackIdAlloc::HOT_PACK_ID.inner())
  }

  #[test]
  fn persistent() {
    let mut alloc = PackIdAlloc::default();
    let next = alloc.next_id();
    // test preset pack_id
    alloc.add_id(PackIdAlloc::HOT_PACK_ID);
    // TODO let pack id support add opts
    // test over next pack_id
    alloc.add_id(PackId::new(next.inner() + 10));
    alloc.add_id(next);
    assert!(!alloc.reuse_pool.contains(&PackIdAlloc::HOT_PACK_ID));
    assert!(!alloc.reuse_pool.iter().any(|item| item > &alloc.next));

    let other_alloc = PackIdAlloc::try_from_string(&alloc.to_string()).expect("should ok");
    assert!(other_alloc.next == alloc.next);
    assert!(other_alloc.reuse_pool == alloc.reuse_pool);
  }
}
