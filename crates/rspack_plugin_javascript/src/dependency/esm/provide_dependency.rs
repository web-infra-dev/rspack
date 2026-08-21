use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsPreset, AsVec},
};
use rspack_core::{
  AsContextDependency, AwaitDependenciesInitFragment, BuildMetaExportsType, Compilation,
  Dependency, DependencyCategory, DependencyCodeGeneration, DependencyId, DependencyLocation,
  DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType, ExportsInfoArtifact,
  InitFragmentKey, InitFragmentStage, ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact,
  NormalInitFragment, ReferencedExport, RuntimeSpec, TemplateContext, TemplateReplaceSource,
  UsedName, create_exports_object_referenced, property_access, to_normal_comment,
};
use rspack_hash::{RspackHash, RspackHasher};
use swc_atoms::Atom;

use super::esm_compatibility_dependency::add_async_module_boundary;

#[cacheable]
#[derive(Debug, Clone)]
pub struct ProvideDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
  request: Atom,
  identifier: String,
  #[cacheable(with=AsVec<AsPreset>)]
  ids: Vec<Atom>,
  range: DependencyRange,
  loc: Option<DependencyLocation>,
}

impl ProvideDependency {
  pub fn new(
    range: DependencyRange,
    request: Atom,
    identifier: String,
    ids: Vec<Atom>,
    loc: Option<DependencyLocation>,
  ) -> Self {
    Self {
      range,
      request,
      loc,
      identifier,
      ids,
      id: DependencyId::new(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for ProvideDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn loc(&self) -> Option<DependencyLocation> {
    self.loc.clone()
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::Provided
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ReferencedExport> {
    if self.ids.is_empty() {
      create_exports_object_referenced()
    } else {
      vec![ReferencedExport::from(self.ids.as_slice())]
    }
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for ProvideDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for ProvideDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ProvideDependencyTemplate::template_type())
  }

  fn update_hash(
    &self,
    hasher: &mut RspackHasher,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) {
    self.identifier.hash(hasher);
    self.ids.hash(hasher);
    // Case: a ProvidePlugin variable is replaced by an inlined const export,
    // e.g. `provided = (__rspack_require("./constants"), 2)`. The generated
    // code embeds the target export's inline literal, so the dependency hash must
    // include that payload and not only the provided identifier/import ids.
    let used_name = compilation
      .get_module_graph()
      .connection_by_dependency_id(&self.id)
      .and_then(|connection| {
        let exports_info = compilation
          .exports_info_artifact
          .get_exports_info_data(connection.module_identifier());
        exports_info.get_used_name(&compilation.exports_info_artifact, runtime, &self.ids)
      });
    if let Some(UsedName::Inlined(inlined)) = used_name {
      inlined.hash(hasher);
    }
  }
}

impl AsContextDependency for ProvideDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ProvideDependencyTemplate;

impl ProvideDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("ProvideDependency")
  }
}

impl DependencyTemplate for ProvideDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ProvideDependency>()
      .expect("ProvideDependencyTemplate should only be used for ProvideDependency");
    let rendered_identifier = source.ensure_generated_top_level_symbol(dep.identifier.clone());

    let TemplateContext {
      compilation,
      module,
      runtime,
      runtime_template,
      init_fragments,
      ..
    } = code_generatable_context;
    let module_graph = compilation.get_module_graph();
    let Some(con) = module_graph.connection_by_dependency_id(&dep.id) else {
      // not find connection, maybe because it's not resolved in make phase, and `bail` is false
      return;
    };

    let exports_info = compilation
      .exports_info_artifact
      .get_exports_info_data(con.module_identifier());
    let used_name =
      exports_info.get_used_name(&compilation.exports_info_artifact, *runtime, &dep.ids);
    let module_raw = runtime_template.module_raw(compilation, dep.id(), dep.request(), dep.weak());
    let is_async =
      ModuleGraph::is_async(&compilation.async_modules_artifact, con.module_identifier());
    let (provided_expr, post_await_expr) = if is_async {
      let post_await_expr = match used_name {
        Some(UsedName::Normal(used_name)) => Some(format!(
          "{rendered_identifier}{}",
          property_access(used_name, 0)
        )),
        Some(UsedName::Inlined(inlined)) => Some(inlined.render(&to_normal_comment(&format!(
          "inlined export {}",
          property_access(&dep.ids, 0)
        )))),
        None => None,
      };
      (module_raw, post_await_expr)
    } else {
      let provided_expr = match used_name {
        Some(UsedName::Normal(used_name)) => {
          format!("{module_raw}{}", property_access(used_name, 0))
        }
        Some(UsedName::Inlined(inlined)) => format!(
          "({}, {})",
          module_raw,
          inlined.render(&to_normal_comment(&format!(
            "inlined export {}",
            property_access(&dep.ids, 0)
          )))
        ),
        None => module_raw,
      };
      (provided_expr, None)
    };

    let mut fragment = NormalInitFragment::new(
      format!("/* provided dependency */ var {rendered_identifier} = {provided_expr};\n"),
      InitFragmentStage::StageProvides,
      1,
      InitFragmentKey::ModuleExternal(format!("provided {}", dep.identifier)),
      None,
    );
    fragment.set_top_level_decl_symbols(vec![dep.identifier.clone().into()]);
    init_fragments.push(Box::new(fragment));
    if is_async {
      if module.build_meta().exports_type() != BuildMetaExportsType::Namespace {
        add_async_module_boundary(init_fragments, compilation, *module, runtime_template, true);
      }
      init_fragments.push(Box::new(AwaitDependenciesInitFragment::new_single(
        rendered_identifier.clone(),
      )));
      if let Some(post_await_expr) = post_await_expr {
        init_fragments.push(Box::new(NormalInitFragment::new(
          format!("{rendered_identifier} = {post_await_expr};\n"),
          InitFragmentStage::StageAsyncESMImports,
          1,
          InitFragmentKey::ModuleExternal(format!("provided async {}", dep.identifier)),
          None,
        )));
      }
    }
    source.replace_with_tracked_used_names(
      dep.range.start,
      dep.range.end,
      rendered_identifier,
      None,
    );
  }
}
