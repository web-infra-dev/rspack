use rspack_collections::IdentifierMap;
use rspack_hash::RspackHashDigest;

use crate::{ModuleIdentifier, RuntimeSpec, RuntimeSpecMap};

#[derive(Debug, Default)]
pub struct CgmHashArtifact {
  module_to_hashes: IdentifierMap<RuntimeSpecMap<RspackHashDigest>>,
}

impl CgmHashArtifact {
  pub fn is_empty(&self) -> bool {
    self.module_to_hashes.is_empty()
  }

  pub fn get(&self, module: &ModuleIdentifier, runtime: &RuntimeSpec) -> Option<&RspackHashDigest> {
    let hashes = self.module_to_hashes.get(module)?;
    hashes.get(runtime)
  }

  pub fn set_hashes(
    &mut self,
    module: ModuleIdentifier,
    hashes: RuntimeSpecMap<RspackHashDigest>,
  ) -> bool {
    if let Some(old) = self.module_to_hashes.get(&module)
      && old == &hashes
    {
      false
    } else {
      self.module_to_hashes.insert(module, hashes);
      true
    }
  }

  pub fn remove(&mut self, module: &ModuleIdentifier) -> Option<RuntimeSpecMap<RspackHashDigest>> {
    self.module_to_hashes.remove(module)
  }
}
