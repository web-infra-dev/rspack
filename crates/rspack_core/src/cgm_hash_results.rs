use rspack_collections::IdentifierMap;
use rspack_hash::RspackHashDigest;

use crate::{ModuleIdentifier, RuntimeSpec, RuntimeSpecMap};

#[derive(Debug, Default)]
pub struct CgmHashResults {
  module_to_hashes: IdentifierMap<RuntimeSpecMap<RspackHashDigest>>,
}

impl CgmHashResults {
  pub fn get(&self, module: &ModuleIdentifier, runtime: &RuntimeSpec) -> Option<&RspackHashDigest> {
    let hashes = self.module_to_hashes.get(module)?;
    hashes.get(runtime)
  }

  pub fn set(&mut self, module: ModuleIdentifier, runtime: RuntimeSpec, hash: RspackHashDigest) {
    let hashes = self
      .module_to_hashes
      .entry(module)
      .or_insert_with(RuntimeSpecMap::new);
    hashes.set(runtime, hash);
  }

  pub fn set_hashes(&mut self, module: ModuleIdentifier, hashes: RuntimeSpecMap<RspackHashDigest>) {
    self.module_to_hashes.insert(module, hashes);
  }
}
