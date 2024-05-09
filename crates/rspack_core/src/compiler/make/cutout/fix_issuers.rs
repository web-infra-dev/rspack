use rustc_hash::FxHashMap as HashMap;

use super::super::MakeArtifact;
use crate::{ModuleIdentifier, ModuleIssuer};

#[derive(Debug, Default)]
pub struct FixIssuers {
  origin_module_issuers: HashMap<ModuleIdentifier, ModuleIssuer>,
}

impl FixIssuers {
  pub fn analyze_force_build_module(
    &mut self,
    artifact: &MakeArtifact,
    module_identifier: &ModuleIdentifier,
  ) {
    let module_graph = artifact.get_module_graph();
    let mgm = module_graph
      .module_graph_module_by_identifier(module_identifier)
      .expect("should have module graph module");
    self
      .origin_module_issuers
      .insert(*module_identifier, mgm.get_issuer().clone());
  }

  pub fn fix_artifact(self, artifact: &mut MakeArtifact) {
    let mut module_graph = artifact.get_module_graph_mut();
    for (id, issuer) in self.origin_module_issuers.into_iter() {
      if let Some(mgm) = module_graph.module_graph_module_by_identifier_mut(&id) {
        mgm.set_issuer(issuer);
      }
    }
  }
}
