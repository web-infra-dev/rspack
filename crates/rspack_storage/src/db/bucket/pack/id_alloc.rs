use itertools::Itertools;

use super::{Error, PackId, Result};

/// Allocator for pack IDs that manages ID generation and reuse.
///
/// Format: "reuse_id1 reuse_id2 ... next_id"
/// - IDs in reuse pool can be reassigned when packs are deleted
/// - Last ID is the next sequential ID to allocate
/// - Pack IDs start at 1, with 0 reserved for the hot pack
#[derive(Debug, PartialEq, Eq)]
pub struct PackIdAlloc {
  next: PackId,
  reuse_pool: Vec<PackId>,
}

impl Default for PackIdAlloc {
  fn default() -> Self {
    Self {
      next: PackId::new(1), // Start at 1, hot pack is 0
      reuse_pool: vec![],
    }
  }
}

impl std::fmt::Display for PackIdAlloc {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let ids = self
      .reuse_pool
      .iter()
      .chain(std::iter::once(&self.next))
      .map(|item| item.to_string())
      .join(" ");
    write!(f, "{}", ids)
  }
}

impl std::str::FromStr for PackIdAlloc {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    let mut ids: Vec<PackId> = s
      .split_whitespace()
      .map(|item| item.parse())
      .collect::<Result<_>>()?;

    let Some(next) = ids.pop() else {
      return Err(Error::InvalidFormat(
        "PackIdAlloc cannot be empty, expected format: 'reuse_ids... next_id'".into(),
      ));
    };

    Ok(Self {
      next,
      reuse_pool: ids,
    })
  }
}

impl PackIdAlloc {
  /// Reserved ID for the hot pack (frequently modified data)
  pub const HOT_PACK_ID: PackId = PackId::new(0);

  /// Allocates the next available pack ID (reuses deleted IDs if available)
  pub fn next_id(&mut self) -> PackId {
    if let Some(id) = self.reuse_pool.pop() {
      return id;
    }
    let next = self.next;
    self.next = next + 1;
    next
  }

  /// Returns a pack ID to the reuse pool when a pack is deleted
  pub fn add_id(&mut self, id: PackId) {
    if id != Self::HOT_PACK_ID && id < self.next {
      self.reuse_pool.push(id);
    }
  }
}

#[cfg(test)]
mod test {
  use super::PackIdAlloc;

  #[test]
  fn test_id_alloc() {
    let mut alloc = PackIdAlloc::default();
    assert!(alloc.next > PackIdAlloc::HOT_PACK_ID);

    let next = alloc.next_id();
    // test preset pack_id
    alloc.add_id(PackIdAlloc::HOT_PACK_ID);
    // test over next pack_id
    alloc.add_id(next + 10);
    alloc.add_id(next);
    assert!(!alloc.reuse_pool.contains(&PackIdAlloc::HOT_PACK_ID));
    assert!(!alloc.reuse_pool.iter().any(|item| item > &alloc.next));

    let other_alloc: PackIdAlloc = alloc.to_string().parse().expect("should ok");
    assert_eq!(other_alloc, alloc);
  }
}
