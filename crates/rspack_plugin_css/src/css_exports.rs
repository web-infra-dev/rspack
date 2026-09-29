use rspack_collections::IdentifierSet;
use rspack_core::{
  ChunkGraph, Compilation, Dependency, DependencyId, Module, ModuleGraph, ModuleIdentifier,
};
use rspack_hash::{RspackHash, RspackHasher};
use rustc_hash::FxHashSet;
use smol_str::SmolStr;

use crate::{
  dependency::{CssIcssExportDependency, CssIcssImportDependency, CssIcssSymbolDependency},
  utils::{export_locals_convention, replace_css_module_id_placeholder},
};

pub(crate) fn css_export_dependency<'a>(
  graph: &'a ModuleGraph,
  id: &DependencyId,
) -> &'a CssIcssExportDependency {
  graph
    .dependency_by_id(id)
    .downcast_ref::<CssIcssExportDependency>()
    .expect("CSS export index must refer to an export definition")
}

pub(crate) fn css_export_name(module: &dyn Module, name: &str) -> SmolStr {
  let info = module.build_info();
  let Some(css) = info.css.as_deref() else {
    return name.into();
  };
  if css.exports.contains_key(name) {
    return name.into();
  }
  let Some(options) = module
    .as_normal_module()
    .and_then(|module| module.get_generator_options())
    .and_then(rspack_core::GeneratorOptions::get_css_module)
  else {
    return name.into();
  };
  let convention = options.exports_convention.unwrap_or_default();
  export_locals_convention(name, convention)
    .into_iter()
    .find(|name| css.exports.contains_key(name.as_str()))
    .map_or_else(|| name.into(), SmolStr::from)
}

fn find_export(module: &dyn Module, name: &str) -> Option<DependencyId> {
  let name = css_export_name(module, name);
  module
    .build_info()
    .css
    .as_deref()?
    .exports
    .get(name.as_str())
    .copied()
}

pub(crate) fn find_css_export_target<'a>(
  compilation: &'a Compilation,
  id: &DependencyId,
) -> Option<&'a dyn Module> {
  compilation
    .get_module_graph()
    .get_module_by_dependency_id(id)
    .map(|module| module.as_ref())
}

pub(crate) fn local_css_identifiers(
  graph: &ModuleGraph,
  roots: impl IntoIterator<Item = DependencyId>,
) -> FxHashSet<DependencyId> {
  let mut pending = roots.into_iter().collect::<Vec<_>>();
  let mut seen = FxHashSet::default();
  let mut locals = FxHashSet::default();
  while let Some(id) = pending.pop() {
    if !seen.insert(id) {
      continue;
    }
    if let Some(dep) = graph
      .dependency_by_id(&id)
      .downcast_ref::<CssIcssExportDependency>()
    {
      if dep.local_ident {
        locals.insert(id);
      }
      pending.extend(
        dep
          .references
          .iter()
          .map(|reference| reference.dependency_id),
      );
      pending.extend(dep.composes.iter().copied());
    }
  }
  locals
}

pub(crate) fn hash_icss_imports(
  compilation: &Compilation,
  module: &dyn Module,
  hasher: &mut RspackHasher,
) {
  let graph = compilation.get_module_graph();
  let mut pending = module
    .get_dependencies()
    .iter()
    .filter_map(|dep| {
      dep
        .downcast_ref::<CssIcssSymbolDependency>()
        .map(|dep| dep.target)
    })
    .collect::<Vec<_>>();
  let mut seen = FxHashSet::default();
  let mut modules = IdentifierSet::default();
  while let Some(id) = pending.pop() {
    if !seen.insert(id) {
      continue;
    }
    let dep = graph.dependency_by_id(&id);
    if let Some(dep) = dep.downcast_ref::<CssIcssExportDependency>() {
      pending.extend(
        dep
          .references
          .iter()
          .map(|reference| reference.dependency_id),
      );
      pending.extend(dep.composes.iter().copied());
    } else if let Some(dep) = dep.downcast_ref::<CssIcssImportDependency>()
      && let Some(target) = find_css_export_target(compilation, &id)
    {
      let identifier = target.identifier();
      if modules.insert(identifier) {
        identifier.hash(hasher);
        ChunkGraph::get_module_id(&compilation.module_ids_artifact, identifier).hash(hasher);
        target.build_info().hash.hash(hasher);
      }
      if let Some(id) = find_export(target, dep.import_name()) {
        pending.push(id);
      }
    }
  }
}

struct CssExportFrame<'a> {
  module: &'a dyn Module,
  dependency: &'a CssIcssExportDependency,
  next_reference: usize,
  cursor: usize,
  tail_emitted: bool,
  next_compose: usize,
  composed: bool,
  resolved: Option<String>,
}

fn append_value(output: &mut Option<String>, value: String, separator: &str) {
  if let Some(output) = output {
    output.push_str(separator);
    output.push_str(&value);
  } else {
    *output = Some(value);
  }
}

// Follow dependency IDs directly. The stack only retains traversal state and
// the output being generated, not a second set of cloned export definitions.
// Only active definitions guard cycles, so shared leaves remain in each branch.
fn resolve_css_dependency_with(
  compilation: &Compilation,
  module: &dyn Module,
  id: DependencyId,
  follow_import: impl Fn(ModuleIdentifier) -> bool,
  separator: &str,
  mut render: impl FnMut(&dyn Module, &str, Option<&CssIcssImportDependency>) -> Option<String>,
) -> Option<String> {
  let graph = compilation.get_module_graph();
  let mut active = FxHashSet::default();
  let mut stack: Vec<CssExportFrame<'_>> = Vec::new();
  let mut pending = Some((module, id, false));
  loop {
    if let Some((owner, id, composed)) = pending.take() {
      let dep = graph.dependency_by_id(&id);
      if let Some(dep) = dep.downcast_ref::<CssIcssExportDependency>() {
        if active.insert(id) {
          stack.push(CssExportFrame {
            module: owner,
            dependency: dep,
            next_reference: 0,
            cursor: 0,
            tail_emitted: false,
            next_compose: 0,
            composed,
            resolved: None,
          });
        }
        continue;
      }
      if let Some(dep) = dep.downcast_ref::<CssIcssImportDependency>() {
        if let Some(target) = find_css_export_target(compilation, &id)
          .filter(|target| follow_import(target.identifier()))
        {
          pending = find_export(target, dep.import_name()).map(|id| (target, id, composed));
          continue;
        }
        if let Some(value) = render(owner, dep.import_name(), Some(dep)) {
          if let Some(parent) = stack.last_mut() {
            if composed
              && parent.resolved.is_some()
              && let Some(space) = render(parent.module, " ", None)
            {
              append_value(&mut parent.resolved, space, separator);
            }
            append_value(&mut parent.resolved, value, separator);
          } else {
            return Some(value);
          }
        }
      }
      continue;
    }
    let frame = stack.last_mut()?;
    if let Some(reference) = frame.dependency.references.get(frame.next_reference) {
      let text = &frame.dependency.value[frame.cursor..reference.range.start as usize];
      if !text.is_empty()
        && let Some(value) = render(frame.module, text, None)
      {
        append_value(&mut frame.resolved, value, separator);
      }
      frame.cursor = reference.range.end as usize;
      frame.next_reference += 1;
      pending = Some((frame.module, reference.dependency_id, false));
      continue;
    }
    if !frame.tail_emitted {
      frame.tail_emitted = true;
      let text = &frame.dependency.value[frame.cursor..];
      if (!text.is_empty() || frame.dependency.references.is_empty())
        && let Some(value) = render(frame.module, text, None)
      {
        append_value(&mut frame.resolved, value, separator);
      }
      continue;
    }
    if let Some(id) = frame.dependency.composes.get(frame.next_compose) {
      frame.next_compose += 1;
      pending = Some((frame.module, *id, true));
      continue;
    }
    let frame = stack.pop().expect("CSS definition frame should exist");
    active.remove(frame.dependency.id());
    if let Some(parent) = stack.last_mut() {
      if let Some(value) = frame.resolved {
        if frame.composed
          && parent.resolved.is_some()
          && let Some(space) = render(parent.module, " ", None)
        {
          append_value(&mut parent.resolved, space, separator);
        }
        append_value(&mut parent.resolved, value, separator);
      }
    } else {
      return frame.resolved;
    }
  }
}

pub(crate) fn resolve_css_dependency(
  compilation: &Compilation,
  module: &dyn Module,
  id: DependencyId,
) -> Option<SmolStr> {
  resolve_css_dependency_with(
    compilation,
    module,
    id,
    |_| true,
    "",
    |module, value, import| {
      import
        .is_none()
        .then(|| replace_css_module_id_placeholder(value, compilation, module).into_owned())
    },
  )
  .map(SmolStr::from)
}

pub(crate) fn resolve_css_export(
  compilation: &Compilation,
  module: &dyn Module,
  name: &str,
) -> Option<SmolStr> {
  resolve_css_dependency(compilation, module, find_export(module, name)?)
}

pub(crate) fn render_css_export(
  compilation: &Compilation,
  module: &dyn Module,
  name: &str,
  mut render: impl FnMut(&dyn Module, &str, Option<&CssIcssImportDependency>) -> Option<String>,
) -> String {
  let Some(id) = find_export(module, name) else {
    return String::new();
  };
  resolve_css_dependency_with(
    compilation,
    module,
    id,
    |target| target == module.identifier(),
    " + ",
    &mut render,
  )
  // A pure cross-module ICSS cycle has no static value. Retain the runtime
  // reference instead of emitting an invalid empty JS expression.
  .or_else(|| resolve_css_dependency_with(compilation, module, id, |_| false, " + ", render))
  .unwrap_or_default()
}
