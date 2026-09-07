use std::sync::atomic::{AtomicU8, Ordering};

use rspack_cacheable::{
  ContextGuard, cacheable,
  with::{As, AsConverter},
};
use rspack_hash::{RspackHash, RspackHasher};
use serde::Serialize;

use crate::{BuildMetaDefaultObject, BuildMetaExportsType};

/// Build metadata with independently mutable flags.
///
/// Setters require only a shared reference. Compilation phases still control
/// when mutation is allowed; changing flags does not invalidate derived results.
/// Cloning copies each flag into independent storage so build-cache entries,
/// concatenated modules, and failed-build recovery retain their own snapshots.
#[cacheable(with=BuildMetaCache)]
#[derive(Debug, Default)]
pub struct BuildMeta {
  strict_esm_module: AtomicU8,
  has_top_level_await: AtomicU8,
  esm: AtomicU8,
  is_css_module: AtomicU8,
  need_id_in_concatenation: AtomicU8,
  exports_type: AtomicU8,
  default_object: AtomicU8,
  side_effect_free: AtomicU8,
}

type BuildMetaCache = As<BuildMetaSnapshot>;

impl Clone for BuildMeta {
  fn clone(&self) -> Self {
    Self {
      strict_esm_module: AtomicU8::new(self.strict_esm_module.load(Ordering::Relaxed)),
      has_top_level_await: AtomicU8::new(self.has_top_level_await.load(Ordering::Relaxed)),
      esm: AtomicU8::new(self.esm.load(Ordering::Relaxed)),
      is_css_module: AtomicU8::new(self.is_css_module.load(Ordering::Relaxed)),
      need_id_in_concatenation: AtomicU8::new(
        self.need_id_in_concatenation.load(Ordering::Relaxed),
      ),
      exports_type: AtomicU8::new(self.exports_type.load(Ordering::Relaxed)),
      default_object: AtomicU8::new(self.default_object.load(Ordering::Relaxed)),
      side_effect_free: AtomicU8::new(self.side_effect_free.load(Ordering::Relaxed)),
    }
  }
}

impl BuildMeta {
  pub fn strict_esm_module(&self) -> bool {
    self.strict_esm_module.load(Ordering::Relaxed) == 2
  }

  pub fn set_strict_esm_module(&self, value: bool) {
    self
      .strict_esm_module
      .store(if value { 2 } else { 1 }, Ordering::Relaxed);
  }

  pub fn has_top_level_await(&self) -> bool {
    self.has_top_level_await.load(Ordering::Relaxed) == 2
  }

  pub fn set_has_top_level_await(&self, value: bool) {
    self
      .has_top_level_await
      .store(if value { 2 } else { 1 }, Ordering::Relaxed);
  }

  pub fn esm(&self) -> bool {
    self.esm.load(Ordering::Relaxed) == 2
  }

  pub fn set_esm(&self, value: bool) {
    self.esm.store(if value { 2 } else { 1 }, Ordering::Relaxed);
  }

  pub fn is_css_module(&self) -> bool {
    self.is_css_module.load(Ordering::Relaxed) == 2
  }

  pub fn set_is_css_module(&self, value: bool) {
    self
      .is_css_module
      .store(if value { 2 } else { 1 }, Ordering::Relaxed);
  }

  pub fn need_id_in_concatenation(&self) -> bool {
    self.need_id_in_concatenation.load(Ordering::Relaxed) == 2
  }

  pub fn set_need_id_in_concatenation(&self, value: bool) {
    self
      .need_id_in_concatenation
      .store(if value { 2 } else { 1 }, Ordering::Relaxed);
  }

  pub fn side_effect_free(&self) -> bool {
    self.side_effect_free.load(Ordering::Relaxed) == 2
  }

  pub fn side_effect_free_value(&self) -> Option<bool> {
    optional_bool(&self.side_effect_free)
  }

  pub fn set_side_effect_free(&self, value: bool) {
    self
      .side_effect_free
      .store(if value { 2 } else { 1 }, Ordering::Relaxed);
  }

  pub fn exports_type(&self) -> BuildMetaExportsType {
    match self.exports_type.load(Ordering::Relaxed) {
      1 => BuildMetaExportsType::Default,
      2 => BuildMetaExportsType::Namespace,
      3 => BuildMetaExportsType::Flagged,
      4 => BuildMetaExportsType::Dynamic,
      _ => BuildMetaExportsType::Unset,
    }
  }

  pub fn set_exports_type(&self, value: BuildMetaExportsType) {
    let value = match value {
      BuildMetaExportsType::Unset => 0,
      BuildMetaExportsType::Default => 1,
      BuildMetaExportsType::Namespace => 2,
      BuildMetaExportsType::Flagged => 3,
      BuildMetaExportsType::Dynamic => 4,
    };
    self.exports_type.store(value, Ordering::Relaxed);
  }

  pub fn clear_exports_type(&self) {
    self.set_exports_type(BuildMetaExportsType::Unset);
  }

  pub fn default_object(&self) -> BuildMetaDefaultObject {
    self
      .default_object_value()
      .unwrap_or(BuildMetaDefaultObject::False)
  }

  fn default_object_value(&self) -> Option<BuildMetaDefaultObject> {
    match self.default_object.load(Ordering::Relaxed) {
      1 => Some(BuildMetaDefaultObject::False),
      2 => Some(BuildMetaDefaultObject::Redirect),
      3 => Some(BuildMetaDefaultObject::RedirectWarn),
      _ => None,
    }
  }

  pub fn set_default_object(&self, value: BuildMetaDefaultObject) {
    let value = match value {
      BuildMetaDefaultObject::False => 1,
      BuildMetaDefaultObject::Redirect => 2,
      BuildMetaDefaultObject::RedirectWarn => 3,
    };
    self.default_object.store(value, Ordering::Relaxed);
  }

  pub fn with_exports_type(self, value: BuildMetaExportsType) -> Self {
    self.set_exports_type(value);
    self
  }

  pub fn with_default_object(self, value: BuildMetaDefaultObject) -> Self {
    self.set_default_object(value);
    self
  }

  /// Restore a previously captured build snapshot, including unset flags.
  pub fn restore(&self, snapshot: &Self) {
    self.strict_esm_module.store(
      snapshot.strict_esm_module.load(Ordering::Relaxed),
      Ordering::Relaxed,
    );
    self.has_top_level_await.store(
      snapshot.has_top_level_await.load(Ordering::Relaxed),
      Ordering::Relaxed,
    );
    self
      .esm
      .store(snapshot.esm.load(Ordering::Relaxed), Ordering::Relaxed);
    self.is_css_module.store(
      snapshot.is_css_module.load(Ordering::Relaxed),
      Ordering::Relaxed,
    );
    self.need_id_in_concatenation.store(
      snapshot.need_id_in_concatenation.load(Ordering::Relaxed),
      Ordering::Relaxed,
    );
    self.exports_type.store(
      snapshot.exports_type.load(Ordering::Relaxed),
      Ordering::Relaxed,
    );
    self.default_object.store(
      snapshot.default_object.load(Ordering::Relaxed),
      Ordering::Relaxed,
    );
    self.side_effect_free.store(
      snapshot.side_effect_free.load(Ordering::Relaxed),
      Ordering::Relaxed,
    );
  }

  fn snapshot(&self) -> BuildMetaSnapshot {
    BuildMetaSnapshot {
      strict_esm_module: optional_bool(&self.strict_esm_module),
      has_top_level_await: optional_bool(&self.has_top_level_await),
      esm: optional_bool(&self.esm),
      is_css_module: optional_bool(&self.is_css_module),
      need_id_in_concatenation: optional_bool(&self.need_id_in_concatenation),
      exports_type: self.exports_type(),
      default_object: self.default_object_value(),
      side_effect_free: optional_bool(&self.side_effect_free),
    }
  }
}

fn optional_bool(value: &AtomicU8) -> Option<bool> {
  match value.load(Ordering::Relaxed) {
    1 => Some(false),
    2 => Some(true),
    _ => None,
  }
}

impl Serialize for BuildMeta {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    self.snapshot().serialize(serializer)
  }
}

impl RspackHash for BuildMeta {
  fn hash(&self, state: &mut RspackHasher) {
    self.snapshot().hash(state);
  }
}

#[cacheable]
#[derive(Serialize, rspack_hash::RspackHash)]
#[serde(rename_all = "camelCase")]
pub struct BuildMetaSnapshot {
  #[serde(skip_serializing_if = "Option::is_none")]
  strict_esm_module: Option<bool>,
  // same as is_async https://github.com/webpack/webpack/blob/3919c844eca394d73ca930e4fc5506fb86e2b094/lib/Module.js#L107
  #[serde(skip_serializing_if = "Option::is_none")]
  has_top_level_await: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  esm: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  is_css_module: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  need_id_in_concatenation: Option<bool>,
  exports_type: BuildMetaExportsType,
  #[serde(skip_serializing_if = "Option::is_none")]
  default_object: Option<BuildMetaDefaultObject>,
  #[serde(skip_serializing_if = "Option::is_none")]
  side_effect_free: Option<bool>,
}

impl AsConverter<BuildMeta> for BuildMetaSnapshot {
  fn serialize(data: &BuildMeta, _guard: &ContextGuard) -> rspack_cacheable::Result<Self> {
    Ok(data.snapshot())
  }

  fn deserialize(self, _guard: &ContextGuard) -> rspack_cacheable::Result<BuildMeta> {
    let meta = BuildMeta::default().with_exports_type(self.exports_type);
    if let Some(value) = self.strict_esm_module {
      meta.set_strict_esm_module(value);
    }
    if let Some(value) = self.has_top_level_await {
      meta.set_has_top_level_await(value);
    }
    if let Some(value) = self.esm {
      meta.set_esm(value);
    }
    if let Some(value) = self.is_css_module {
      meta.set_is_css_module(value);
    }
    if let Some(value) = self.need_id_in_concatenation {
      meta.set_need_id_in_concatenation(value);
    }
    if let Some(value) = self.side_effect_free {
      meta.set_side_effect_free(value);
    }
    if let Some(value) = self.default_object {
      meta.set_default_object(value);
    }
    Ok(meta)
  }
}
