use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::atoms::Atom;

use super::super::MakeArtifact;
use crate::{AsyncDependenciesBlockIdentifier, GroupOptions, ModuleGraph, ModuleIdentifier};

#[derive(Debug, Default, Eq, PartialEq, Clone)]
struct ModuleDeps {
  // child module identifier of current module
  child_modules: HashMap<ModuleIdentifier, HashSet<Atom>>,
  // blocks in current module
  module_blocks: Vec<(AsyncDependenciesBlockIdentifier, Option<GroupOptions>)>,
}

impl ModuleDeps {
  fn from_module(module_graph: &ModuleGraph, module_identifier: &ModuleIdentifier) -> Self {
    let mut res = Self::default();
    let module = module_graph
      .module_by_identifier(module_identifier)
      .expect("should have module");

    let deps = module.get_dependencies();
    let mut child_deps: HashMap<ModuleIdentifier, HashSet<Atom>> = Default::default();

    for dep_id in deps {
      let dep = module_graph
        .dependency_by_id(dep_id)
        .expect("should have dependency");

      let Some(conn) = module_graph.connection_by_dependency(dep_id) else {
        continue;
      };
      let identifier = conn.module_identifier();
      let ids = child_deps.entry(*identifier).or_default();

      if matches!(
        dep.dependency_type(),
        crate::DependencyType::EsmImportSpecifier
      ) {
        let dep_ids = dep.get_ids(module_graph);
        ids.extend(dep_ids.into_iter());
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
  origin_module_deps: HashMap<ModuleIdentifier, ModuleDeps>,
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

  pub fn fix_artifact(self, artifact: &mut MakeArtifact) {
    let module_graph = &artifact.get_module_graph();
    if self.origin_module_deps.is_empty() {
      // origin_module_deps empty means no force_build_module and no file changed
      // this only happens when build from entry
      artifact.has_module_graph_change = true;
      return;
    }
    // if artifact.has_module_graph_change is true, no need to recalculate
    if !artifact.has_module_graph_change {
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
}

#[cfg(test)]
mod t {
  use std::borrow::Cow;

  use itertools::Itertools;
  use rspack_error::{impl_empty_diagnosable_trait, Diagnostic, Result};
  use rspack_identifier::Identifiable;
  use rspack_macros::impl_source_map_config;
  use rspack_sources::Source;
  use rspack_util::source_map::SourceMapKind;

  use crate::{
    compiler::make::cutout::has_module_graph_change::ModuleDeps, AsContextDependency, BuildInfo,
    BuildMeta, CodeGenerationResult, Compilation, ConcatenationScope, Context, DependenciesBlock,
    Dependency, DependencyId, DependencyTemplate, ExportsInfoId, FactoryMeta, Module,
    ModuleDependency, ModuleGraph, ModuleGraphModule, ModuleGraphPartial, ModuleIdentifier,
    ModuleType, RuntimeSpec, SourceType,
  };

  #[derive(Debug, Clone)]
  struct TestDep {
    ids: Vec<&'static str>,
    id: DependencyId,
  }

  impl TestDep {
    fn new(ids: Vec<&'static str>) -> Self {
      Self {
        ids,
        id: DependencyId::new(),
      }
    }
  }

  impl AsContextDependency for TestDep {}

  impl Dependency for TestDep {
    fn dependency_type(&self) -> &crate::DependencyType {
      &crate::DependencyType::EsmImportSpecifier
    }

    fn dependency_debug_name(&self) -> &'static str {
      "test dep"
    }

    fn id(&self) -> &DependencyId {
      &self.id
    }

    fn get_ids(&self, _mg: &ModuleGraph) -> Vec<swc_core::atoms::Atom> {
      self
        .ids
        .iter()
        .map(|id| id.to_string().into())
        .collect_vec()
    }
  }

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
  }

  impl ModuleDependency for TestDep {
    fn request(&self) -> &str {
      ""
    }
  }

  #[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
  #[impl_source_map_config]
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

    fn get_dependencies(&self) -> &[DependencyId] {
      &self.deps
    }
  }

  impl Identifiable for TestModule {
    fn identifier(&self) -> rspack_identifier::Identifier {
      self.id
    }
  }

  impl_empty_diagnosable_trait!(TestModule);

  impl Module for TestModule {
    fn module_type(&self) -> &ModuleType {
      todo!()
    }

    fn source_types(&self) -> &[SourceType] {
      todo!()
    }

    fn get_diagnostics(&self) -> Vec<Diagnostic> {
      todo!()
    }

    fn original_source(&self) -> Option<&dyn Source> {
      todo!()
    }

    fn readable_identifier(&self, _context: &Context) -> Cow<str> {
      todo!()
    }

    fn size(&self, _source_type: &SourceType) -> f64 {
      todo!()
    }

    fn factory_meta(&self) -> Option<&FactoryMeta> {
      todo!()
    }

    fn set_factory_meta(&mut self, _factory_meta: FactoryMeta) {
      todo!()
    }

    fn build_info(&self) -> Option<&BuildInfo> {
      todo!()
    }

    fn set_build_info(&mut self, _build_info: BuildInfo) {
      todo!()
    }

    fn build_meta(&self) -> Option<&BuildMeta> {
      todo!()
    }

    fn set_build_meta(&mut self, _build_meta: BuildMeta) {
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
  }

  #[test]
  #[allow(unused_results, clippy::unwrap_used)]
  fn test_module_deps() {
    let mut partial = ModuleGraphPartial::default();
    let mut mg = ModuleGraph::new(vec![], Some(&mut partial));

    let dep1 = Box::new(TestDep::new(vec!["foo"]));
    let dep1_id = *dep1.id();
    let module_orig = Box::new(TestModule::new("app", vec![dep1_id]));
    let module_orig_id = module_orig.identifier();
    let module1 = Box::new(TestModule::new("lib_foo", vec![dep1_id]));
    let module1_id = module1.id;

    mg.add_module(module_orig);
    mg.add_module_graph_module(ModuleGraphModule::new(module_orig_id, ExportsInfoId::new()));
    mg.add_module(module1);
    mg.add_module_graph_module(ModuleGraphModule::new(module1_id, ExportsInfoId::new()));
    mg.add_dependency(dep1);
    mg.set_resolved_module(Some(module_orig_id), dep1_id, module1_id)
      .unwrap();

    let module_deps_1 = ModuleDeps::from_module(&mg, &module1_id);
    let module_deps_2 = ModuleDeps::from_module(&mg, &module1_id);

    assert_eq!(module_deps_1, module_deps_2);

    let dep2 = Box::new(TestDep::new(vec!["bar"]));
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
