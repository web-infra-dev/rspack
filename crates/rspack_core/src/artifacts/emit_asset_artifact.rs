use rustc_hash::FxHashMap as HashMap;

use crate::{ArtifactExt, incremental::IncrementalPasses};

/// Artifact for tracking emitted asset versions
///
/// This artifact stores the version information of emitted assets to enable
/// incremental compilation. By comparing asset versions between compilations,
/// we can skip emitting assets that haven't changed.
///
/// Similar to webpack's `comparedForEmitAssets` but version-based.
#[derive(Debug, Default)]
pub struct EmitAssetArtifact {
  /// Maps asset filename to its version string
  /// The key is the filename, the value is the version from AssetInfo
  emitted_asset_versions: HashMap<String, String>,
}

impl ArtifactExt for EmitAssetArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::EMIT_ASSETS;
}

impl EmitAssetArtifact {
  pub fn new() -> Self {
    Self::default()
  }

  /// Get the version of an emitted asset
  pub fn get_version(&self, filename: &str) -> Option<&String> {
    self.emitted_asset_versions.get(filename)
  }

  /// Set the version of an asset
  pub fn set_version(&mut self, filename: String, version: String) {
    self.emitted_asset_versions.insert(filename, version);
  }

  /// Check if an asset version has changed
  pub fn has_version_changed(&self, filename: &str, new_version: &str) -> bool {
    match self.emitted_asset_versions.get(filename) {
      Some(old_version) => old_version != new_version || new_version.is_empty(),
      None => true,
    }
  }

  /// Clear all stored versions
  pub fn clear(&mut self) {
    self.emitted_asset_versions.clear();
  }

  /// Get all emitted asset versions
  pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
    self.emitted_asset_versions.iter()
  }

  /// Check if empty
  pub fn is_empty(&self) -> bool {
    self.emitted_asset_versions.is_empty()
  }
}
