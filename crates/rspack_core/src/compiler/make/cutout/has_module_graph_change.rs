use indexmap::IndexMap;
use rspack_collections::IdentifierMap;
use rustc_hash::FxHashSet as HashSet;
use swc_core::atoms::Atom;

use super::super::MakeArtifact;
use crate::{AsyncDependenciesBlockIdentifier, GroupOptions, ModuleGraph, ModuleIdentifier};

#[derive(Debug, Default, Clone)]
struct ModuleDeps {
  // child module identifier of current module
  child_modules: IndexMap<ModuleIdentifier, HashSet<Atom>>,
  // blocks in current module
  module_blocks: Vec<(AsyncDependenciesBlockIdentifier, Option<GroupOptions>)>,
}

impl std::cmp::PartialEq for ModuleDeps {
  fn eq(&self, other: &Self) -> bool {
    // check imports order
    if !self.child_modules.iter().eq(other.child_modules.iter()) {
      return false;
    }
    /* TODO:
     * we should check order in imported ids as well,
     * different ids may comes from different modules
     * after reexports optimized, but check the order
     * of imported ids can break some usual sceneries
     * like turns `import {A, B} from 'foo'` into
     * `import {B, A} from 'foo'`, chunk graph cannot
     * change, this is usual in development, we will
     * drop this module deps support after newIncremental
     * is stabilized
     */

    self.module_blocks == other.module_blocks
  }
}

impl ModuleDeps {
  fn from_module(module_graph: &ModuleGraph, module_identifier: &ModuleIdentifier) -> Self {
    let mut res = Self::default();
    let module = module_graph
      .module_by_identifier(module_identifier)
      .expect("should have module");

    let deps = module.get_dependencies();
    let mut child_deps: IndexMap<ModuleIdentifier, HashSet<Atom>> = Default::default();

    for dep_id in deps {
      let dep = module_graph
        .dependency_by_id(dep_id)
        .expect("should have dependency");

      let Some(conn) = module_graph.connection_by_dependency_id(dep_id) else {
        continue;
      };
      let identifier = conn.module_identifier();
      let ids = child_deps.entry(*identifier).or_default();

      if matches!(
        dep.dependency_type(),
        crate::DependencyType::EsmImportSpecifier
      ) {
        // TODO: remove Dependency::get_ids once incremental build chunk graph is stable.
        let dep_ids = dep._get_ids(module_graph);
        ids.extend(dep_ids.iter().cloned());
      }
    }

    res.child_modules = child_deps;
    for block_id in module.get_blocks() {
      let block = module_graph
        .block_by_id(block_id)
        .expect("should have block");
      res
        .module_blocks
        .push((*block_id, block.get_group_options().cloned()));
    }

    res
  }
}

#[derive(Debug, Default)]
pub struct HasModuleGraphChange {
  disabled: bool,
  expect_built_modules_len: usize,
  origin_module_deps: IdentifierMap<ModuleDeps>,
}

impl HasModuleGraphChange {
  pub fn analyze_force_build_module(
    &mut self,
    artifact: &MakeArtifact,
    module_identifier: &ModuleIdentifier,
  ) {
    let module_graph = &artifact.get_module_graph();
    self.origin_module_deps.insert(
      *module_identifier,
      ModuleDeps::from_module(module_graph, module_identifier),
    );
  }

  pub fn analyze_artifact(&mut self, artifact: &MakeArtifact) {
    if artifact.has_module_graph_change {
      self.disabled = true;
      return;
    }

    self.expect_built_modules_len = artifact.built_modules.len();
    for module_identifier in self.origin_module_deps.keys() {
      if !artifact.built_modules.contains(module_identifier) {
        self.expect_built_modules_len += 1;
      }
    }
  }

  pub fn fix_artifact(self, artifact: &mut MakeArtifact) {
    let module_graph = &artifact.get_module_graph();
    if self.disabled {
      return;
    }
    if artifact.built_modules.len() != self.expect_built_modules_len
      || artifact.built_modules.len() != artifact.revoked_modules.len()
    {
      // contain unexpected module built
      artifact.has_module_graph_change = true;
      return;
    }

    for (module_identifier, module_deps) in self.origin_module_deps {
      if module_graph
        .module_by_identifier(&module_identifier)
        .is_none()
        || ModuleDeps::from_module(module_graph, &module_identifier) != module_deps
      {
        artifact.has_module_graph_change = true;
        return;
      }
    }
  }
}

#[cfg(test)]
mod t {
  use std::borrow::Cow;

  use rspack_cacheable::{cacheable, cacheable_dyn, with::Skip};
  use rspack_collections::Identifiable;
  use rspack_error::{impl_empty_diagnosable_trait, Result};
  use rspack_macros::impl_source_map_config;
  use rspack_sources::BoxSource;
  use rspack_util::{atom::Atom, source_map::SourceMapKind};

  use crate::{
    compiler::make::cutout::has_module_graph_change::ModuleDeps, AffectType, AsContextDependency,
    BuildInfo, BuildMeta, CodeGenerationResult, Compilation, ConcatenationScope, Context,
    DependenciesBlock, Dependency, DependencyId, DependencyTemplate, ExportsInfo, FactorizeInfo,
    FactoryMeta, Module, ModuleDependency, ModuleGraph, ModuleGraphModule, ModuleGraphPartial,
    ModuleIdentifier, ModuleType, RuntimeSpec, SourceType,
  };

  #[cacheable]
  #[derive(Debug, Clone)]
  struct TestDep {
    #[cacheable(with=Skip)]
    ids: Vec<Atom>,
    id: DependencyId,
    factorize_info: FactorizeInfo,
  }

  impl TestDep {
    fn new(ids: Vec<Atom>) -> Self {
      Self {
        ids,
        id: DependencyId::new(),
        factorize_info: Default::default(),
      }
    }
  }

  impl AsContextDependency for TestDep {}

  #[cacheable_dyn]
  impl Dependency for TestDep {
    fn dependency_type(&self) -> &crate::DependencyType {
      &crate::DependencyType::EsmImportSpecifier
    }

    fn id(&self) -> &DependencyId {
      &self.id
    }

    fn _get_ids<'a>(&'a self, _mg: &'a ModuleGraph) -> &'a [Atom] {
      &self.ids
    }

    fn could_affect_referencing_module(&self) -> AffectType {
      AffectType::True
    }
  }

  #[cacheable_dyn]
  impl DependencyTemplate for TestDep {
    fn apply(
      &self,
      _source: &mut crate::TemplateReplaceSource,
      _code_generatable_context: &mut crate::TemplateContext,
    ) {
      todo!()
    }

    fn dependency_id(&self) -> Option<DependencyId> {
      None
    }

    fn update_hash(
      &self,
      _hasher: &mut dyn std::hash::Hasher,
      _compilation: &Compilation,
      _runtime: Option<&RuntimeSpec>,
    ) {
      todo!()
    }
  }

  #[cacheable_dyn]
  impl ModuleDependency for TestDep {
    fn request(&self) -> &str {
      ""
    }

    fn factorize_info(&self) -> &FactorizeInfo {
      unreachable!()
    }

    fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
      unreachable!()
    }
  }

  #[impl_source_map_config]
  #[cacheable]
  #[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
  struct TestModule {
    pub(crate) id: ModuleIdentifier,
    deps: Vec<DependencyId>,
  }

  impl TestModule {
    fn new(id: &str, deps: Vec<DependencyId>) -> Self {
      Self {
        id: id.into(),
        deps,
        source_map_kind: SourceMapKind::empty(),
      }
    }
  }

  impl DependenciesBlock for TestModule {
    fn add_block_id(&mut self, _block: crate::AsyncDependenciesBlockIdentifier) {
      todo!()
    }

    fn get_blocks(&self) -> &[crate::AsyncDependenciesBlockIdentifier] {
      &[]
    }

    fn add_dependency_id(&mut self, dependency: DependencyId) {
      self.deps.push(dependency);
    }

    fn remove_dependency_id(&mut self, dependency: DependencyId) {
      self.deps.retain(|dep| dep != &dependency);
    }

    fn get_dependencies(&self) -> &[DependencyId] {
      &self.deps
    }
  }

  impl Identifiable for TestModule {
    fn identifier(&self) -> rspack_collections::Identifier {
      self.id
    }
  }

  impl_empty_diagnosable_trait!(TestModule);

  #[cacheable_dyn]
  impl Module for TestModule {
    fn module_type(&self) -> &ModuleType {
      todo!()
    }

    fn source_types(&self) -> &[SourceType] {
      todo!()
    }

    fn source(&self) -> Option<&BoxSource> {
      todo!()
    }

    fn readable_identifier(&self, _context: &Context) -> Cow<str> {
      todo!()
    }

    fn size(&self, _source_type: Option<&SourceType>, _compilation: Option<&Compilation>) -> f64 {
      todo!()
    }

    fn factory_meta(&self) -> Option<&FactoryMeta> {
      todo!()
    }

    fn set_factory_meta(&mut self, _factory_meta: FactoryMeta) {
      todo!()
    }

    fn build_info(&self) -> &BuildInfo {
      todo!()
    }

    fn build_info_mut(&mut self) -> &mut BuildInfo {
      todo!()
    }

    fn build_meta(&self) -> &BuildMeta {
      todo!()
    }

    fn build_meta_mut(&mut self) -> &mut BuildMeta {
      todo!()
    }

    fn code_generation(
      &self,
      _compilation: &Compilation,
      _runtime: Option<&RuntimeSpec>,
      _concatenation_scope: Option<ConcatenationScope>,
    ) -> Result<CodeGenerationResult> {
      todo!()
    }

    fn update_hash(
      &self,
      _hasher: &mut dyn std::hash::Hasher,
      _compilation: &Compilation,
      _runtime: Option<&RuntimeSpec>,
    ) -> Result<()> {
      todo!()
    }
  }

  #[test]
  #[allow(unused_results, clippy::unwrap_used)]
  fn test_module_deps() {
    let mut partial = ModuleGraphPartial::default();
    let mut mg = ModuleGraph::new(vec![], Some(&mut partial));

    let dep1 = Box::new(TestDep::new(vec!["foo".into()]));
    let dep1_id = *dep1.id();
    let module_orig = Box::new(TestModule::new("app", vec![dep1_id]));
    let module_orig_id = module_orig.identifier();
    let module1 = Box::new(TestModule::new("lib_foo", vec![dep1_id]));
    let module1_id = module1.id;

    mg.add_module(module_orig);
    mg.add_module_graph_module(ModuleGraphModule::new(module_orig_id, ExportsInfo::new()));
    mg.add_module(module1);
    mg.add_module_graph_module(ModuleGraphModule::new(module1_id, ExportsInfo::new()));
    mg.add_dependency(dep1);
    mg.set_resolved_module(Some(module_orig_id), dep1_id, module1_id)
      .unwrap();

    let module_deps_1 = ModuleDeps::from_module(&mg, &module1_id);
    let module_deps_2 = ModuleDeps::from_module(&mg, &module1_id);

    assert_eq!(module_deps_1, module_deps_2);

    let dep2 = Box::new(TestDep::new(vec!["bar".into()]));
    let dep2_id = *dep2.id();
    let module_orig: &mut TestModule = mg
      .module_by_identifier_mut(&module_orig_id)
      .expect("should have module")
      .downcast_mut()
      .expect("should be test module");
    module_orig.add_dependency_id(dep2_id);

    mg.add_dependency(dep2);
    mg.set_resolved_module(Some(module_orig_id), dep2_id, module1_id)
      .unwrap();

    let module_deps_3 = ModuleDeps::from_module(&mg, &module_orig_id);
    assert_ne!(module_deps_3, module_deps_1);
  }
}
