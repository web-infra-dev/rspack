/// Memory accounting for a dense, id-indexed storage table.
///
/// Used to observe how module graph storage scales across rebuilds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DenseStorageStats {
  /// Upper bound of ids the table can currently address without growing.
  pub id_space: usize,
  /// Number of occupied slots.
  pub live: usize,
  /// Number of occupied slots stored in pages rather than in a flat prefix.
  pub paged: usize,
  /// Number of allocated slots, including empty ones.
  pub slots: usize,
  /// Approximate heap bytes held by the table itself, excluding heap data owned by values.
  pub bytes: usize,
}

impl std::ops::Add for DenseStorageStats {
  type Output = Self;

  fn add(self, rhs: Self) -> Self {
    Self {
      id_space: self.id_space.max(rhs.id_space),
      live: self.live + rhs.live,
      paged: self.paged + rhs.paged,
      slots: self.slots + rhs.slots,
      bytes: self.bytes + rhs.bytes,
    }
  }
}
