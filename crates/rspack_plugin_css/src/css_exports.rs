use std::borrow::Cow;

use rspack_collections::IdentifierSet;
use rspack_core::{
  ChunkGraph, Compilation, CssExport, CssExportPart, Dependency, DependencyId, FreezeReadGuard,
  Module, ModuleIdentifier,
};
use rspack_hash::{RspackHash, RspackHasher};
use rspack_util::fx_hash::FxIndexSet;
use rustc_hash::FxHashSet;
use smol_str::SmolStr;

use crate::{dependency::CssIcssSymbolDependency, utils::replace_css_module_id_placeholder};

pub(crate) fn hash_icss_imports(
  compilation: &Compilation,
  module: &dyn Module,
  hasher: &mut RspackHasher,
) {
  let mut pending = module
    .get_dependencies()
    .iter()
    .filter_map(|dependency| dependency.downcast_ref::<CssIcssSymbolDependency>())
    .flat_map(|dependency| &dependency.value().parts)
    .filter_map(|value| {
      find_css_export_target(
        compilation,
        module,
        value.from.as_deref()?,
        value.id.as_ref(),
      )
    })
    .collect::<Vec<_>>();
  let mut seen = IdentifierSet::default();
  while let Some(target) = pending.pop() {
    let identifier = target.identifier();
    if !seen.insert(identifier) {
      continue;
    }
    // Static replacements depend on the imported content, including transitive
    // reexports. Track build hashes without expanding the values a second time.
    identifier.hash(hasher);
    ChunkGraph::get_module_id(&compilation.module_ids_artifact, identifier).hash(hasher);
    let info = target.build_info();
    info.hash.hash(hasher);
    if let Some(css) = info.css.as_deref() {
      pending.extend(
        css
          .exports
          .values()
          .flatten()
          .flat_map(|export| &export.parts)
          .filter_map(|value| {
            find_css_export_target(
              compilation,
              target,
              value.from.as_deref()?,
              value.id.as_ref(),
            )
          }),
      );
    }
  }
}

fn dependency_request(dependency: &dyn Dependency) -> Option<&str> {
  dependency
    .as_module_dependency()
    .map(|dep| dep.request())
    .or_else(|| dependency.as_context_dependency().map(|dep| dep.request()))
}

pub(crate) fn find_css_export_target<'a>(
  compilation: &'a Compilation,
  module: &dyn Module,
  request: &str,
  id: Option<&DependencyId>,
) -> Option<&'a dyn Module> {
  let module_graph = compilation.get_module_graph();
  id.and_then(|id| module_graph.get_module_by_dependency_id(id))
    .or_else(|| {
      module.get_dependencies().iter().find_map(|dependency| {
        (dependency_request(dependency.as_ref()) == Some(request))
          .then(|| module_graph.get_module_by_dependency_id(dependency.id()))?
      })
    })
    .map(|module| module.as_ref())
}

struct CssExportFrame<'a, T> {
  module: &'a dyn Module,
  name: SmolStr,
  exports: FreezeReadGuard<'a, FxIndexSet<CssExport>>,
  next_index: usize,
  next_part: usize,
  current: Vec<T>,
  resolved: Vec<T>,
}

impl<'a, T> CssExportFrame<'a, T> {
  fn new(module: &'a dyn Module, name: &str) -> Option<Self> {
    let exports = module
      .build_info()
      .try_map(|info| info.css.as_deref()?.exports.get(name))?;
    Some(Self {
      module,
      name: name.into(),
      exports,
      next_index: 0,
      next_part: 0,
      current: Vec::new(),
      resolved: Vec::new(),
    })
  }
}

// Resolve one export on demand. Only the active path is tracked: a shared leaf
// must still appear in every independent composition branch that references it.
fn resolve_css_exports<T>(
  compilation: &Compilation,
  module: &dyn Module,
  name: &str,
  should_follow: impl Fn(ModuleIdentifier) -> bool,
  leaf: impl Fn(&dyn Module, &CssExportPart) -> Option<T>,
) -> Vec<T> {
  let Some(root) = CssExportFrame::new(module, name) else {
    return Vec::new();
  };
  let mut active = FxHashSet::from_iter([(module.identifier(), SmolStr::new(name))]);
  let mut stack = vec![root];
  while let Some(frame) = stack.last_mut() {
    let Some(export) = frame.exports.get_index(frame.next_index) else {
      let frame = stack.pop().expect("CSS export frame should exist");
      active.remove(&(frame.module.identifier(), frame.name));
      if let Some(parent) = stack.last_mut() {
        parent.current.extend(frame.resolved);
      } else {
        return frame.resolved;
      }
      continue;
    };
    // Compositions add a space between values; compound ICSS parts concatenate
    // verbatim, retaining the original punctuation and whitespace.
    let Some(part) = export.parts.get(frame.next_part) else {
      if !frame.current.is_empty() {
        if !frame.resolved.is_empty()
          && let Some(space) = leaf(
            frame.module,
            &CssExportPart {
              ident: " ".into(),
              from: None,
              id: None,
            },
          )
        {
          frame.resolved.push(space);
        }
        frame.resolved.append(&mut frame.current);
      }
      frame.next_index += 1;
      frame.next_part = 0;
      continue;
    };
    frame.next_part += 1;
    let target = part.from.as_deref().and_then(|request| {
      find_css_export_target(compilation, frame.module, request, part.id.as_ref())
    });
    if let Some(target) = target.filter(|target| should_follow(target.identifier())) {
      let key = (target.identifier(), part.ident.clone());
      if !active.contains(&key)
        && let Some(child) = CssExportFrame::new(target, &part.ident)
      {
        active.insert(key);
        stack.push(child);
      }
    } else if let Some(value) = leaf(frame.module, part) {
      frame.current.push(value);
    }
  }
  Vec::new()
}

pub(crate) fn resolve_css_export_parts(
  compilation: &Compilation,
  module: &dyn Module,
  name: &str,
) -> Vec<CssExportPart> {
  let mut parts = resolve_css_exports(
    compilation,
    module,
    name,
    |target| target == module.identifier(),
    |_, part| Some(part.clone()),
  );
  // A pure ICSS cycle has no concrete value. Preserve its runtime resolution.
  if parts.is_empty()
    && let Some(exports) = module
      .build_info()
      .css
      .as_deref()
      .and_then(|css| css.exports.get(name))
  {
    for export in exports {
      if !parts.is_empty() {
        parts.push(CssExportPart {
          ident: " ".into(),
          from: None,
          id: None,
        });
      }
      parts.extend_from_slice(&export.parts);
    }
  }
  parts
}

pub(crate) fn resolve_css_export(
  compilation: &Compilation,
  module: &dyn Module,
  name: &str,
) -> Option<SmolStr> {
  let values = resolve_css_exports(
    compilation,
    module,
    name,
    |_| true,
    |module, part| {
      part.from.is_none().then(|| {
        match replace_css_module_id_placeholder(&part.ident, compilation, module) {
          Cow::Borrowed(_) => part.ident.clone(),
          Cow::Owned(value) => SmolStr::from(value),
        }
      })
    },
  );
  match values.as_slice() {
    [] => None,
    [value] => Some(value.clone()),
    values => Some(values.iter().map(SmolStr::as_str).collect()),
  }
}
