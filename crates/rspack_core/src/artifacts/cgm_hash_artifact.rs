use rspack_collections::IdentifierMap;
use rspack_hash::RspackHashDigest;

use crate::{
  ArtifactExt, ModuleIdentifier, RuntimeSpec, RuntimeSpecMap, incremental::IncrementalPasses,
};

#[derive(Debug, Default)]
pub struct CgmHashArtifact {
  module_to_hashes: IdentifierMap<RuntimeSpecMap<RspackHashDigest>>,
  module_to_code_generation_hash: IdentifierMap<RspackHashDigest>,
}

impl ArtifactExt for CgmHashArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::MODULES_HASHES;
}

impl FromIterator<(ModuleIdentifier, RuntimeSpecMap<RspackHashDigest>)> for CgmHashArtifact {
  fn from_iter<T: IntoIterator<Item = (ModuleIdentifier, RuntimeSpecMap<RspackHashDigest>)>>(
    iter: T,
  ) -> Self {
    Self {
      module_to_hashes: IdentifierMap::from_iter(iter),
      module_to_code_generation_hash: Default::default(),
    }
  }
}

impl CgmHashArtifact {
  pub fn is_empty(&self) -> bool {
    self.module_to_hashes.is_empty()
  }

  pub fn iter(
    &self,
  ) -> impl Iterator<Item = (&ModuleIdentifier, &RuntimeSpecMap<RspackHashDigest>)> {
    self.module_to_hashes.iter()
  }

  pub fn code_generation_hashes_iter(
    &self,
  ) -> impl Iterator<Item = (&ModuleIdentifier, &RspackHashDigest)> {
    self.module_to_code_generation_hash.iter()
  }

  pub fn get_runtime_map(
    &self,
    module: &ModuleIdentifier,
  ) -> Option<&RuntimeSpecMap<RspackHashDigest>> {
    self.module_to_hashes.get(module)
  }

  pub fn get_code_generation_hash(&self, module: &ModuleIdentifier) -> Option<&RspackHashDigest> {
    self.module_to_code_generation_hash.get(module)
  }

  pub fn get(&self, module: &ModuleIdentifier, runtime: &RuntimeSpec) -> Option<&RspackHashDigest> {
    let hashes = self.module_to_hashes.get(module)?;
    hashes.get(runtime)
  }

  pub fn set_hashes(
    &mut self,
    module: ModuleIdentifier,
    hashes: RuntimeSpecMap<RspackHashDigest>,
    code_generation_hash: Option<RspackHashDigest>,
  ) -> bool {
    let hashes_unchanged = self
      .module_to_hashes
      .get(&module)
      .is_some_and(|old| old == &hashes);
    let code_generation_hash_unchanged =
      self.module_to_code_generation_hash.get(&module) == code_generation_hash.as_ref();
    if hashes_unchanged && code_generation_hash_unchanged {
      false
    } else {
      self.module_to_hashes.insert(module, hashes);
      if let Some(code_generation_hash) = code_generation_hash {
        self
          .module_to_code_generation_hash
          .insert(module, code_generation_hash);
      } else {
        self.module_to_code_generation_hash.remove(&module);
      }
      true
    }
  }

  pub fn set_code_generation_hash(&mut self, module: ModuleIdentifier, hash: RspackHashDigest) {
    self.module_to_code_generation_hash.insert(module, hash);
  }

  pub fn remove(&mut self, module: &ModuleIdentifier) -> Option<RuntimeSpecMap<RspackHashDigest>> {
    self.module_to_code_generation_hash.remove(module);
    self.module_to_hashes.remove(module)
  }

  pub fn clear(&mut self) {
    self.module_to_hashes.clear();
    self.module_to_code_generation_hash.clear();
  }
}
