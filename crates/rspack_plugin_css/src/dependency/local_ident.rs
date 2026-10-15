use std::hash::Hash;

use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsPreset, AsVec},
};
use rspack_core::{
  AsContextDependency, AsModuleDependency, Compilation, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyTemplate, DependencyTemplateType,
  DependencyType, ExportNameOrSpec, ExportSpec, ExportsInfoArtifact, ExportsOfExportsSpec,
  ExportsSpec, RuntimeSpec, TemplateContext, TemplateReplaceSource,
};
use rspack_hash::RspackHasher;
use rspack_intern::Atom;

use crate::{css_syntax::escape_identifier, utils::replace_css_module_id_placeholder};

#[cacheable]
#[derive(Debug)]
pub struct CssLocalIdentDependency {
  id: DependencyId,
  /// Interned hashed ident shared with every use of this class.
  #[cacheable(with=AsPreset)]
  local_ident: Atom,
  /// Interned export names. Archived as the same string list as the previous `Vec<String>`.
  #[cacheable(with=AsVec<AsPreset>)]
  convention_names: Vec<Atom>,
  start: u32,
  end: u32,
}

impl CssLocalIdentDependency {
  pub fn new(local_ident: String, convention_names: Vec<String>, start: u32, end: u32) -> Self {
    Self {
      id: DependencyId::new(),
      local_ident: Atom::from(local_ident),
      convention_names: convention_names.into_iter().map(Atom::from).collect(),
      start,
      end,
    }
  }
}

#[cacheable_dyn]
impl Dependency for CssLocalIdentDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CssLocalIdent
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssLocalIdent
  }

  fn get_exports(
    &self,
    _mg: &rspack_core::ModuleGraph,
    _module_graph_cache: &rspack_core::ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
  ) -> Option<ExportsSpec> {
    Some(ExportsSpec {
      exports: ExportsOfExportsSpec::Names(
        self
          .convention_names
          .iter()
          .map(|name| {
            ExportNameOrSpec::ExportSpec(ExportSpec {
              name: name.as_str().into(),
              can_mangle: Some(true),
              ..Default::default()
            })
          })
          .collect(),
      ),
      ..Default::default()
    })
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for CssLocalIdentDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CssLocalIdentDependencyTemplate::template_type())
  }

  fn update_hash(
    &self,
    hasher: &mut RspackHasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    // Hash the text, not `Atom`'s cached hash, so module hashes stay stable.
    self.local_ident.as_str().hash(hasher);
  }
}

impl AsContextDependency for CssLocalIdentDependency {}
impl AsModuleDependency for CssLocalIdentDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CssLocalIdentDependencyTemplate;

impl CssLocalIdentDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::CssLocalIdent)
  }
}

impl DependencyTemplate for CssLocalIdentDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<CssLocalIdentDependency>()
      .expect("CssLocalIdentDependencyTemplate should be used for CssLocalIdentDependency");

    let local_ident = replace_css_module_id_placeholder(
      &dep.local_ident,
      code_generatable_context.compilation,
      code_generatable_context.module,
    );

    source.replace(
      dep.start,
      dep.end,
      escape_identifier(&local_ident).into_owned(),
      None,
    );
  }
}
