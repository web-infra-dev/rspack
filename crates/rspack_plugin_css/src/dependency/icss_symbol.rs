use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsModuleDependency, Compilation, CssExport, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, Module, ModuleIdentifier, TemplateContext,
  TemplateReplaceSource,
};
use rustc_hash::FxHashSet;

use crate::utils::replace_css_module_id_placeholder;

#[cacheable]
#[derive(Debug, Clone)]
pub enum CssIcssSymbolValue {
  Literal(String),
  Import {
    local_name: String,
    import_name: String,
    request: String,
  },
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct CssIcssSymbolDependency {
  id: DependencyId,
  value: CssIcssSymbolValue,
  range: DependencyRange,
}

impl CssIcssSymbolDependency {
  pub fn new(value: CssIcssSymbolValue, range: DependencyRange) -> Self {
    Self {
      id: DependencyId::new(),
      value,
      range,
    }
  }
}

#[cacheable_dyn]
impl Dependency for CssIcssSymbolDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CssExport
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssIcssSymbol
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for CssIcssSymbolDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CssIcssSymbolDependencyTemplate::template_type())
  }
}

impl AsContextDependency for CssIcssSymbolDependency {}
impl AsModuleDependency for CssIcssSymbolDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CssIcssSymbolDependencyTemplate;

impl CssIcssSymbolDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::CssIcssSymbol)
  }
}

impl DependencyTemplate for CssIcssSymbolDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<CssIcssSymbolDependency>()
      .expect("CssIcssSymbolDependencyTemplate should be used for CssIcssSymbolDependency");

    let value = match &dep.value {
      CssIcssSymbolValue::Literal(value) => Some(value.clone()),
      CssIcssSymbolValue::Import {
        local_name,
        import_name,
        request,
      } => resolve_icss_import(
        code_generatable_context.compilation,
        code_generatable_context.module,
        local_name,
        import_name,
        request,
      ),
    };

    if let Some(value) = value {
      source.replace(dep.range.start, dep.range.end, value, None);
    }
  }
}

fn resolve_icss_import(
  compilation: &Compilation,
  module: &dyn Module,
  local_name: &str,
  import_name: &str,
  request: &str,
) -> Option<String> {
  let module_graph = compilation.get_module_graph();
  let imported_module = module.get_dependencies().iter().find_map(|id| {
    let dependency = module_graph.dependency_by_id(id);
    let dependency_request = dependency
      .as_module_dependency()
      .map(|d| d.request())
      .or_else(|| dependency.as_context_dependency().map(|d| d.request()));
    if dependency_request == Some(request) {
      module_graph.module_graph_module_by_dependency_id(id)
    } else {
      None
    }
  })?;
  let imported_module = module_graph.module_by_identifier(&imported_module.module_identifier)?;
  resolve_css_export_value(
    compilation,
    imported_module.as_ref(),
    import_name,
    &mut FxHashSet::from_iter([(module.identifier(), local_name.to_string())]),
  )
}

fn resolve_css_export_value(
  compilation: &Compilation,
  module: &dyn Module,
  name: &str,
  seen: &mut FxHashSet<(ModuleIdentifier, String)>,
) -> Option<String> {
  if !seen.insert((module.identifier(), name.to_string())) {
    return None;
  }

  let css_build_info = module.build_info().css.as_deref()?;
  let exports = css_build_info.exports()?;
  let css_exports = exports.get(name)?;
  let mut resolved = Vec::with_capacity(css_exports.len());
  for CssExport { ident, from, .. } in css_exports {
    if let Some(from) = from {
      let Some(value) = resolve_icss_import(compilation, module, ident, ident, from) else {
        continue;
      };
      resolved.push(value);
    } else {
      resolved.push(replace_css_module_id_placeholder(ident, compilation, module).to_string());
    }
  }

  if resolved.is_empty() {
    None
  } else {
    Some(resolved.join(" "))
  }
}
