use rspack_cacheable::{
  cacheable,
  utils::OwnedOrRef,
  with::{AsOption, AsOwned},
};

use super::Etag;

/// Filesystem representation for values exclusively owned by the caller.
/// Serialization borrows the live value; restoration creates a new owned value.
#[cacheable]
pub(super) struct StoredOwnedCacheEntry<'a, T> {
  pub etag: Option<Etag>,
  /// None invalidates a previously stored entry when serialization fails.
  #[cacheable(with=AsOption<AsOwned>)]
  pub value: Option<OwnedOrRef<'a, T>>,
}
