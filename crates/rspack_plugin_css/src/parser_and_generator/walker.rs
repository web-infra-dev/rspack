use rspack_collections::IdentifierSet;
use rspack_core::{Compilation, CssExportType, Dependency, DependencyId, DependencyType, Module};
use rspack_error::Result;
use rustc_hash::FxHashSet;

use crate::dependency::{CssIcssExportDependency, CssIcssImportDependency, CssImportDependency};

#[derive(Default, Clone, Copy)]
pub(super) struct CssExportRequirements {
  pub css: bool,
  pub js: bool,
}

pub(super) struct CssModuleGraphWalker<'g> {
  compilation: &'g Compilation,
  active_exports: FxHashSet<DependencyId>,
  active_imports: IdentifierSet,
}

impl<'g> CssModuleGraphWalker<'g> {
  pub(super) fn new(compilation: &'g Compilation) -> Self {
    Self {
      compilation,
      active_exports: Default::default(),
      active_imports: Default::default(),
    }
  }

  pub(super) fn resolve_css_export_dependency<S: CssModuleGraphSink<'g>>(
    &mut self,
    sink: &mut S,
    id: DependencyId,
    requirements: CssExportRequirements,
    follow_self_imports: bool,
  ) -> Result<S::ExportValue> {
    if self.active_exports.contains(&id) || !sink.should_visit_export(id) {
      return Ok(S::ExportValue::default());
    }
    self.active_exports.insert(id);
    let dependency = self.compilation.get_module_graph().dependency_by_id(&id);
    let result = if let Some(import) = dependency.downcast_ref::<CssIcssImportDependency>() {
      sink.resolve_icss_import(self, import, requirements, follow_self_imports)
    } else if let Some(export) = dependency.downcast_ref::<CssIcssExportDependency>() {
      resolve_definition(self, sink, export, requirements, follow_self_imports)
    } else {
      Ok(S::ExportValue::default())
    };
    self.active_exports.remove(&id);
    result
  }

  fn resolve_css_import(
    &self,
    id: &DependencyId,
  ) -> Option<(&'g CssImportDependency, &'g dyn Module)> {
    let graph = self.compilation.get_module_graph();
    let dependency = graph
      .dependency_by_id(id)
      .downcast_ref::<CssImportDependency>()?;
    let module = graph.get_module_by_dependency_id(id)?;
    Some((dependency, module.as_ref()))
  }

  fn css_import_parent(&self, dependency: &CssImportDependency) -> Option<&'g dyn Module> {
    let graph = self.compilation.get_module_graph();
    let id = graph.get_parent_module(dependency.id())?;
    graph.module_by_identifier(id).map(AsRef::as_ref)
  }

  pub(super) fn is_css_import_inlined_style(
    &self,
    dependency: &CssImportDependency,
    module: &dyn Module,
  ) -> bool {
    let parent_has_conditions = self.css_import_parent(dependency).is_some_and(|parent| {
      parent
        .build_info()
        .css
        .as_deref()
        .is_some_and(|css| css.has_render_conditions())
    });
    parent_has_conditions
      || dependency.render_conditions().next().is_some()
      || crate::utils::css_module_export_type(module) != Some(CssExportType::Style)
  }

  /// Visit source imports in parser order. The sink controls global dedup and
  /// whether an import is embedded; recursively embedded CSS includes its own
  /// source imports before the sink consumes its output.
  pub(super) fn walk_css_imports<S: CssModuleImportSink<'g>>(
    &mut self,
    sink: &mut S,
    module: &dyn Module,
    include_import_sources: bool,
  ) -> Result<()> {
    let root = module.identifier();
    if !self.active_imports.insert(root) {
      return Ok(());
    }
    let result = self.walk_css_import_dependencies(sink, module, include_import_sources);
    self.active_imports.remove(&root);
    result
  }

  fn walk_css_import_dependencies<S: CssModuleImportSink<'g>>(
    &mut self,
    sink: &mut S,
    module: &dyn Module,
    include_import_sources: bool,
  ) -> Result<()> {
    for dependency in module.get_dependencies() {
      if dependency.dependency_type() != &DependencyType::CssImport {
        continue;
      }
      let Some((dependency, module)) = self.resolve_css_import(dependency.id()) else {
        continue;
      };
      let identifier = module.identifier();
      if !self.active_imports.insert(identifier) {
        continue;
      }
      let result =
        if let Some(state) = sink.enter_import(self, dependency, module, include_import_sources)? {
          self.walk_css_import_dependencies(sink, module, true)?;
          sink.leave_import(dependency, module, state)
        } else {
          Ok(())
        };
      self.active_imports.remove(&identifier);
      result?;
    }
    Ok(())
  }
}

pub(super) trait CssModuleGraphSink<'g>: Send {
  type ExportValue: Default + Send;

  fn should_visit_export(&mut self, _id: DependencyId) -> bool {
    true
  }
  fn reference(&mut self, _from: DependencyId, _to: DependencyId) {}
  fn compose(&mut self, _from: DependencyId, _to: DependencyId) {}
  fn local(
    &mut self,
    _export: &'g CssIcssExportDependency,
    _requirements: CssExportRequirements,
  ) -> Self::ExportValue {
    Self::ExportValue::default()
  }
  fn append_literal(
    &self,
    _value: &mut Self::ExportValue,
    _text: &'g str,
    _requirements: CssExportRequirements,
  ) {
  }
  fn append_reference(&self, _value: &mut Self::ExportValue, _reference: Self::ExportValue) {}
  fn append_compose(&self, _value: &mut Self::ExportValue, _compose: Self::ExportValue) {}
  fn resolve_icss_import(
    &mut self,
    walker: &mut CssModuleGraphWalker<'g>,
    import: &'g CssIcssImportDependency,
    requirements: CssExportRequirements,
    follow_self_imports: bool,
  ) -> Result<Self::ExportValue>;
}

pub(super) trait CssModuleImportSink<'g>: Send {
  type State: Send;

  fn enter_import(
    &mut self,
    walker: &mut CssModuleGraphWalker<'g>,
    dependency: &'g CssImportDependency,
    module: &'g dyn Module,
    include_import_sources: bool,
  ) -> Result<Option<Self::State>>;
  fn leave_import(
    &mut self,
    dependency: &'g CssImportDependency,
    module: &'g dyn Module,
    state: Self::State,
  ) -> Result<()>;
}

fn resolve_definition<'g, S: CssModuleGraphSink<'g>>(
  walker: &mut CssModuleGraphWalker<'g>,
  sink: &mut S,
  export: &'g CssIcssExportDependency,
  requirements: CssExportRequirements,
  follow_self_imports: bool,
) -> Result<S::ExportValue> {
  let mut resolved = if export.local_ident {
    sink.local(export, requirements)
  } else {
    let mut resolved = S::ExportValue::default();
    let mut cursor = 0;
    for reference in &export.references {
      sink.reference(*export.id(), reference.dependency_id);
      let text = &export.value[cursor..reference.range.start as usize];
      if (requirements.css || requirements.js) && !text.is_empty() {
        sink.append_literal(&mut resolved, text, requirements);
      }
      let value = walker.resolve_css_export_dependency(
        sink,
        reference.dependency_id,
        requirements,
        follow_self_imports,
      )?;
      sink.append_reference(&mut resolved, value);
      cursor = reference.range.end as usize;
    }
    let tail = &export.value[cursor..];
    if (requirements.css || requirements.js) && (!tail.is_empty() || export.references.is_empty()) {
      sink.append_literal(&mut resolved, tail, requirements);
    }
    resolved
  };
  for id in &export.composes {
    sink.compose(*export.id(), *id);
    let value =
      walker.resolve_css_export_dependency(sink, *id, requirements, follow_self_imports)?;
    sink.append_compose(&mut resolved, value);
  }
  Ok(resolved)
}
