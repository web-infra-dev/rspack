use std::borrow::Cow;

use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsCacheable, AsMap},
};
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  ChunkGraph, CodeGenerationData, CodeGenerationDataItem, Compilation, CssExport, Dependency,
  DependencyId, FreezeReadGuard, Module, ModuleIdentifier,
};
use rspack_hash::{RspackHash, RspackHasher};
use rspack_util::fx_hash::FxIndexSet;
use rustc_hash::{FxHashMap, FxHashSet};
use smol_str::SmolStr;

use crate::utils::replace_css_module_id_placeholder;

#[cacheable]
#[derive(Debug, Default)]
struct CodeGenerationDataCssExports {
  // Cache complete root resolutions only. Intermediate values can depend on
  // which ancestors are active when a composition cycle is encountered.
  #[cacheable(with=AsMap<AsCacheable, AsMap>)]
  values: IdentifierMap<FxHashMap<String, Option<String>>>,
}

#[cacheable_dyn]
impl CodeGenerationDataItem for CodeGenerationDataCssExports {}

pub(crate) fn hash_icss_imports(
  compilation: &Compilation,
  module: &dyn Module,
  hasher: &mut RspackHasher,
) {
  let build_info = module.build_info();
  let Some(css) = build_info.css.as_deref() else {
    return;
  };
  let mut pending = css
    .icss_symbols
    .keys()
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
      pending.extend(css.exports.values().flatten().filter_map(|value| {
        find_css_export_target(
          compilation,
          target,
          value.from.as_deref()?,
          value.id.as_ref(),
        )
      }));
    }
  }
}

pub(crate) fn dependency_request(dependency: &dyn Dependency) -> Option<&str> {
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

struct CssExportFrame<'a> {
  module: &'a dyn Module,
  name: SmolStr,
  exports: FreezeReadGuard<'a, FxIndexSet<CssExport>>,
  next_index: usize,
}

impl<'a> CssExportFrame<'a> {
  fn new(module: &'a dyn Module, name: &str) -> Option<Self> {
    let exports = module
      .build_info()
      .try_map(|info| info.css.as_deref()?.exports.get(name))?;
    Some(Self {
      module,
      name: name.into(),
      exports,
      next_index: 0,
    })
  }
}

fn walk_css_exports(
  compilation: &Compilation,
  module: &dyn Module,
  name: &str,
  should_follow: impl Fn(ModuleIdentifier) -> bool,
  mut visit: impl FnMut(&dyn Module, &CssExport),
) {
  let Some(root) = CssExportFrame::new(module, name) else {
    return;
  };
  let mut active = FxHashSet::from_iter([(module.identifier(), root.name.clone())]);
  let mut stack = vec![root];
  while let Some(frame) = stack.last_mut() {
    let Some(export) = frame.exports.get_index(frame.next_index) else {
      active.remove(&(frame.module.identifier(), frame.name.clone()));
      stack.pop();
      continue;
    };
    frame.next_index += 1;
    let target = export.from.as_deref().and_then(|request| {
      find_css_export_target(compilation, frame.module, request, export.id.as_ref())
    });
    if let Some(target) = target.filter(|target| should_follow(target.identifier())) {
      let key = (target.identifier(), export.ident.clone());
      // Only the active path cuts cycles. Independent branches must retain
      // repeated leaves and their original order.
      if !active.contains(&key)
        && let Some(child) = CssExportFrame::new(target, &export.ident)
      {
        active.insert(key);
        stack.push(child);
      }
    } else {
      visit(frame.module, export);
    }
  }
}

pub(crate) fn expand_self_referencing_exports<'a>(
  compilation: &Compilation,
  module: &dyn Module,
  name: &str,
  elements: &'a FxIndexSet<CssExport>,
) -> impl Iterator<Item = Cow<'a, CssExport>> + use<'a> {
  let module_identifier = module.identifier();
  let has_self_reference = elements.iter().any(|export| {
    export.from.as_deref().is_some_and(|request| {
      find_css_export_target(compilation, module, request, export.id.as_ref())
        .is_some_and(|target| target.identifier() == module_identifier)
    })
  });
  let expanded = if has_self_reference {
    let mut expanded = Vec::new();
    walk_css_exports(
      compilation,
      module,
      name,
      |target| target == module_identifier,
      // Retain the leaf beyond the traversal's metadata guard. External
      // references keep their dependency identity for runtime reads.
      |_, export| expanded.push(export.clone()),
    );
    // A pure ICSS cycle has no concrete value. Preserve its runtime resolution.
    (!expanded.is_empty()).then_some(expanded)
  } else {
    None
  };
  let use_original = expanded.is_none();
  expanded.into_iter().flatten().map(Cow::Owned).chain(
    elements
      .iter()
      .filter(move |_| use_original)
      .map(Cow::Borrowed),
  )
}

pub(crate) fn resolve_css_export(
  compilation: &Compilation,
  module: &dyn Module,
  name: &str,
  data: &mut CodeGenerationData,
) -> Option<String> {
  if let Some(value) = data
    .get::<CodeGenerationDataCssExports>()
    .and_then(|data| data.values.get(&module.identifier()))
    .and_then(|exports| exports.get(name))
  {
    return value.clone();
  }
  let mut resolved: Option<String> = None;
  walk_css_exports(
    compilation,
    module,
    name,
    |_| true,
    |module, export| {
      if export.from.is_some() {
        return;
      }
      let value = replace_css_module_id_placeholder(&export.ident, compilation, module);
      if let Some(resolved) = &mut resolved {
        resolved.push(' ');
        resolved.push_str(&value);
      } else {
        resolved = Some(value.into_owned());
      }
    },
  );
  if !data.contains::<CodeGenerationDataCssExports>() {
    data.insert(CodeGenerationDataCssExports::default());
  }
  data
    .get_mut::<CodeGenerationDataCssExports>()
    .expect("CSS export resolutions should be initialized")
    .values
    .entry(module.identifier())
    .or_default()
    .insert(name.to_owned(), resolved.clone());
  resolved
}
