use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  AsContextDependency, AsModuleDependency, ConnectionState, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, ExportNameOrSpec, ExportSpec, ExportsInfoArtifact,
  ExportsOfExportsSpec, ExportsSpec, ModuleGraph, ModuleGraphCacheArtifact,
  SideEffectsStateArtifact, TemplateContext, TemplateReplaceSource, UsedName, property_name,
};

use super::OBJECT_PROTOTYPE_METHODS;
use crate::Atom;

#[cacheable]
#[derive(Debug, Clone, Copy)]
pub enum CommonJsObjectExportKind {
  KeyValue,
  Shorthand,
  Getter,
  Setter,
  Method,
  AsyncMethod,
  GeneratorMethod,
  AsyncGeneratorMethod,
}

impl CommonJsObjectExportKind {
  fn unused_prefix(self) -> Option<&'static str> {
    match self {
      Self::Getter | Self::Setter | Self::Method => Some("...void (function "),
      Self::AsyncMethod => Some("...void (async function "),
      Self::GeneratorMethod => Some("...void (function* "),
      Self::AsyncGeneratorMethod => Some("...void (async function* "),
      Self::KeyValue | Self::Shorthand => None,
    }
  }
}

#[cacheable]
#[derive(Debug)]
pub struct CommonJsObjectExportDependency {
  id: DependencyId,
  range: DependencyRange,
  key_range: DependencyRange,
  value_range: DependencyRange,
  #[cacheable(with=AsPreset)]
  name: Atom,
  kind: CommonJsObjectExportKind,
  pure: bool,
}

impl CommonJsObjectExportDependency {
  pub fn new(
    range: DependencyRange,
    key_range: DependencyRange,
    value_range: DependencyRange,
    name: Atom,
    kind: CommonJsObjectExportKind,
    pure: bool,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      range,
      key_range,
      value_range,
      name,
      kind,
      pure,
    }
  }
}

#[cacheable_dyn]
impl Dependency for CommonJsObjectExportDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CjsObjectExports
  }

  fn get_exports(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
  ) -> Option<ExportsSpec> {
    Some(ExportsSpec {
      exports: ExportsOfExportsSpec::Names(vec![ExportNameOrSpec::ExportSpec(ExportSpec {
        can_mangle: Some(
          self.name != "__esModule" && !OBJECT_PROTOTYPE_METHODS.contains(&self.name.as_str()),
        ),
        name: self.name.clone(),
        ..Default::default()
      })]),
      ..Default::default()
    })
  }

  fn get_module_evaluation_side_effects_state(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _side_effects_state_artifact: &SideEffectsStateArtifact,
    _module_chain: &mut IdentifierSet,
    _connection_state_cache: &mut IdentifierMap<ConnectionState>,
  ) -> ConnectionState {
    ConnectionState::Active(false)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }
}

impl AsModuleDependency for CommonJsObjectExportDependency {}
impl AsContextDependency for CommonJsObjectExportDependency {}

#[cacheable_dyn]
impl DependencyCodeGeneration for CommonJsObjectExportDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CommonJsObjectExportDependencyTemplate::template_type())
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CommonJsObjectExportDependencyTemplate;

impl CommonJsObjectExportDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::CjsObjectExports)
  }
}

impl DependencyTemplate for CommonJsObjectExportDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<CommonJsObjectExportDependency>()
      .expect(
        "CommonJsObjectExportDependencyTemplate should only be used for CommonJsObjectExportDependency",
      );

    // The CommonJS interop marker must remain stable and present even when it
    // is not referenced directly.
    if dep.name == "__esModule" {
      return;
    }

    let TemplateContext {
      compilation,
      module,
      runtime,
      ..
    } = code_generatable_context;
    let exports_info = compilation
      .exports_info_artifact
      .get_exports_info_data(&module.identifier());
    let used = exports_info.get_used_name(
      &compilation.exports_info_artifact,
      *runtime,
      std::slice::from_ref(&dep.name),
    );

    let Some(UsedName::Normal(used)) = used else {
      if !compilation.options.output.environment.supports_spread() {
        return;
      }

      if let Some(prefix) = dep.kind.unused_prefix() {
        // Keep the function body intact so replacements for dependencies in
        // the body continue to target valid source ranges.
        source.replace(
          dep.range.start,
          dep.value_range.start,
          prefix.to_string(),
          None,
        );
        source.insert_static(dep.value_range.end, ")", None);
      } else if matches!(dep.kind, CommonJsObjectExportKind::Shorthand) {
        source.replace(
          dep.range.start,
          dep.range.end,
          format!("...void ({})", dep.name),
          None,
        );
      } else {
        if dep.pure {
          // Spreading null adds nothing. Keep the value text in a dead branch
          // so nested dependency replacements continue to target valid ranges.
          source.replace_static(
            dep.range.start,
            dep.value_range.start,
            "...(/* unused pure expression */ null && (",
            None,
          );
          source.replace_static(dep.value_range.end, dep.range.end, "))", None);
          return;
        }
        // Preserve evaluation of an unused data property's impure value.
        source.replace_static(dep.range.start, dep.value_range.start, "...void (", None);
        source.replace_static(dep.value_range.end, dep.range.end, ")", None);
      }
      return;
    };

    let [used] = used.as_slice() else {
      return;
    };
    if used == &dep.name {
      return;
    }

    let used = property_name(used).expect("export name should be a valid property name");
    if matches!(dep.kind, CommonJsObjectExportKind::Shorthand) {
      source.replace(
        dep.key_range.start,
        dep.key_range.end,
        format!("{used}: {}", dep.name),
        None,
      );
    } else {
      source.replace(
        dep.key_range.start,
        dep.key_range.end,
        used.into_owned(),
        None,
      );
    }
  }
}
