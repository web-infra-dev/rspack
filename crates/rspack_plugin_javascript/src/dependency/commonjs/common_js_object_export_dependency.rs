use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  AsContextDependency, AsModuleDependency, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, ExportNameOrSpec, ExportSpec, ExportsInfoArtifact,
  ExportsOfExportsSpec, ExportsSpec, ModuleGraph, ModuleGraphCacheArtifact, TemplateContext,
  TemplateReplaceSource, UsedName, property_name,
};
use swc_atoms::Atom;

use super::OBJECT_PROTOTYPE_METHODS;

#[cacheable]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
  fn function_drop_head(self) -> Option<&'static str> {
    match self {
      Self::Getter | Self::Setter | Self::Method => Some("...void (function "),
      Self::AsyncMethod => Some("...void (async function "),
      Self::GeneratorMethod => Some("...void (function* "),
      Self::AsyncGeneratorMethod => Some("...void (async function* "),
      Self::KeyValue | Self::Shorthand => None,
    }
  }

  pub fn is_function(self) -> bool {
    self.function_drop_head().is_some()
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
  value_has_leading_parenthesis: bool,
}

impl CommonJsObjectExportDependency {
  pub fn new(
    range: DependencyRange,
    key_range: DependencyRange,
    value_range: DependencyRange,
    name: Atom,
    kind: CommonJsObjectExportKind,
    pure: bool,
    value_has_leading_parenthesis: bool,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      range,
      key_range,
      value_range,
      name,
      kind,
      pure,
      value_has_leading_parenthesis,
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
    &DependencyType::CjsExports
  }

  fn get_exports(
    &self,
    _mg: &ModuleGraph,
    _mg_cache: &ModuleGraphCacheArtifact,
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
    DependencyTemplateType::Custom("CommonJsObjectExportDependency")
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
      .expect("CommonJsObjectExportDependencyTemplate should only be used for CommonJsObjectExportDependency");

    // The CommonJS/ESM interop marker is observable even when no importer reads it.
    if dep.name == "__esModule" {
      return;
    }

    let exports_info = code_generatable_context
      .compilation
      .exports_info_artifact
      .get_exports_info_data(&code_generatable_context.module.identifier());
    let names = [dep.name.clone()];
    let used = exports_info.get_used_name(
      &code_generatable_context.compilation.exports_info_artifact,
      code_generatable_context.runtime,
      &names,
    );

    let Some(UsedName::Normal(used)) = used else {
      // Object spread is needed to preserve evaluation while contributing no key.
      if !code_generatable_context
        .compilation
        .options
        .output
        .environment
        .supports_spread()
      {
        return;
      }

      if let Some(head) = dep.kind.function_drop_head() {
        // Keep the body intact so nested dependency ranges remain valid.
        source.replace_static(dep.range.start, dep.value_range.start, head, None);
        source.insert_static(dep.value_range.end, ")", None);
      } else if dep.kind == CommonJsObjectExportKind::Shorthand {
        source.replace(
          dep.range.start,
          dep.range.end,
          format!("...void ({})", dep.name),
          None,
        );
      } else if dep.pure {
        source.replace_static(
          dep.range.start,
          dep.value_range.start,
          "...(/* unused pure expression */ null && (",
          None,
        );
        if dep.value_has_leading_parenthesis {
          source.insert_static(dep.value_range.end, ")", None);
        } else {
          source.replace_static(dep.value_range.end, dep.range.end, "))", None);
        }
      } else {
        source.replace_static(dep.range.start, dep.value_range.start, "...void (", None);
        if !dep.value_has_leading_parenthesis {
          source.replace_static(dep.value_range.end, dep.range.end, ")", None);
        }
      }
      return;
    };

    let Some(used_name) = used.last() else {
      return;
    };
    if used_name == &dep.name {
      return;
    }
    let rendered_name = property_name(used_name).expect("property name rendering should not fail");
    if dep.kind == CommonJsObjectExportKind::Shorthand {
      source.replace(
        dep.key_range.start,
        dep.key_range.end,
        format!("{rendered_name}: {}", dep.name),
        None,
      );
    } else {
      source.replace(
        dep.key_range.start,
        dep.key_range.end,
        rendered_name.into_owned(),
        None,
      );
    }
  }
}
