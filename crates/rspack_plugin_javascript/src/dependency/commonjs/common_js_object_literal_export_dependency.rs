use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  AsContextDependency, AsModuleDependency, ConnectionState, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, ExportNameOrSpec, ExportSpec, ExportsInfoArtifact,
  ExportsOfExportsSpec, ExportsSpec, ModuleGraph, ModuleGraphCacheArtifact,
  SideEffectsStateArtifact, TemplateContext, TemplateReplaceSource, UsedName, property_name,
};

use crate::{Atom, dependency::commonjs::OBJECT_PROTOTYPE_METHODS};

#[cacheable]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommonJsObjectLiteralExportKind {
  KeyValue,
  Shorthand,
  Get,
  Set,
  Method,
  AsyncMethod,
  GeneratorMethod,
  AsyncGeneratorMethod,
}

impl CommonJsObjectLiteralExportKind {
  pub const fn is_function_property(self) -> bool {
    self.drop_head().is_some()
  }

  const fn drop_head(self) -> Option<&'static str> {
    match self {
      Self::KeyValue | Self::Shorthand => None,
      Self::Get | Self::Set | Self::Method => Some("...void (function "),
      Self::AsyncMethod => Some("...void (async function "),
      Self::GeneratorMethod => Some("...void (function* "),
      Self::AsyncGeneratorMethod => Some("...void (async function* "),
    }
  }
}

#[cacheable]
#[derive(Debug)]
pub struct CommonJsObjectLiteralExportDependency {
  id: DependencyId,
  range: DependencyRange,
  key_range: DependencyRange,
  value_range: DependencyRange,
  #[cacheable(with=AsPreset)]
  name: Atom,
  kind: CommonJsObjectLiteralExportKind,
  pure: bool,
  fixed: bool,
}

impl CommonJsObjectLiteralExportDependency {
  pub fn new(
    range: DependencyRange,
    key_range: DependencyRange,
    value_range: DependencyRange,
    name: Atom,
    kind: CommonJsObjectLiteralExportKind,
    pure: bool,
    fixed: bool,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      range,
      key_range,
      value_range,
      name,
      kind,
      pure,
      fixed,
    }
  }
}

#[cacheable_dyn]
impl Dependency for CommonJsObjectLiteralExportDependency {
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
    &DependencyType::CjsObjectLiteralExport
  }

  fn get_exports(
    &self,
    _mg: &ModuleGraph,
    _mg_cache: &ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
  ) -> Option<ExportsSpec> {
    Some(ExportsSpec {
      exports: ExportsOfExportsSpec::Names(vec![ExportNameOrSpec::ExportSpec(ExportSpec {
        can_mangle: Some(!self.fixed && !OBJECT_PROTOTYPE_METHODS.contains(&self.name.as_str())),
        name: self.name.clone(),
        ..Default::default()
      })]),
      ..Default::default()
    })
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
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
}

impl AsModuleDependency for CommonJsObjectLiteralExportDependency {}

#[cacheable_dyn]
impl DependencyCodeGeneration for CommonJsObjectLiteralExportDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CommonJsObjectLiteralExportDependencyTemplate::template_type())
  }
}

impl AsContextDependency for CommonJsObjectLiteralExportDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CommonJsObjectLiteralExportDependencyTemplate;

impl CommonJsObjectLiteralExportDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::CjsObjectLiteralExport)
  }
}

impl DependencyTemplate for CommonJsObjectLiteralExportDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<CommonJsObjectLiteralExportDependency>()
      .expect(
        "CommonJsObjectLiteralExportDependencyTemplate should only be used for CommonJsObjectLiteralExportDependency",
      );
    if dep.fixed {
      return;
    }

    let TemplateContext {
      compilation,
      module,
      runtime,
      ..
    } = code_generatable_context;

    let used = compilation
      .exports_info_artifact
      .get_exports_info_data(&module.identifier())
      .get_used_name(
        &compilation.exports_info_artifact,
        *runtime,
        std::slice::from_ref(&dep.name),
      );

    let (range, key_range, value_range) = (dep.range, dep.key_range, dep.value_range);
    match used {
      None => {
        if !compilation.options.output.environment.supports_spread() {
          return;
        }
        // Only the text around the value is replaced, so replacements made by
        // dependencies nested in the value stay valid.
        if let Some(head) = dep.kind.drop_head() {
          source.replace_static(range.start, value_range.start, head, None);
          source.insert_static(value_range.end, ")", None);
        } else if dep.kind == CommonJsObjectLiteralExportKind::Shorthand {
          source.replace(
            range.start,
            range.end,
            format!("...void ({})", dep.name),
            None,
          );
        } else {
          let (head, tail) = if dep.pure {
            ("...(/* unused pure expression */ null && (", "))")
          } else {
            ("...void (", ")")
          };
          source.replace_static(range.start, value_range.start, head, None);
          source.replace_static(value_range.end, range.end, tail, None);
        }
      }
      Some(UsedName::Normal(used)) => {
        let Some(used) = used.last().filter(|used| **used != dep.name) else {
          return;
        };
        let key = property_name(used).expect("should render property name");
        let content = if dep.kind == CommonJsObjectLiteralExportKind::Shorthand {
          format!("{key}: {}", dep.name)
        } else {
          key.into_owned()
        };
        source.replace(key_range.start, key_range.end, content, None);
      }
      Some(UsedName::Inlined(_)) => {}
    }
  }
}
