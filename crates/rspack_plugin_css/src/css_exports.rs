use std::sync::Arc;

use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsCacheable, AsMap, AsPreset, AsVec},
};
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  ChunkGraph, CodeGenerationData, CodeGenerationDataItem, Compilation, CssExport, CssExportPart,
  Dependency, DependencyId, FreezeReadGuard, Module, ModuleIdentifier, SourceType,
};
use rspack_hash::{RspackHash, RspackHasher};
use rspack_util::fx_hash::FxIndexSet;
use rustc_hash::{FxHashMap, FxHashSet};
use smol_str::SmolStr;

use crate::{dependency::CssIcssSymbolDependency, utils::replace_css_module_id_placeholder};

#[cacheable]
#[derive(Debug, Default)]
struct CodeGenerationDataCssExports {
  #[cacheable(with=AsMap)]
  modules: IdentifierMap<CssModuleExports>,
  #[cacheable(with=AsMap)]
  targets: IdentifierMap<CssExportTargets>,
  #[cacheable(with=AsMap<AsCacheable, AsMap<AsPreset>>)]
  values: IdentifierMap<FxHashMap<SmolStr, Option<String>>>,
}

#[cacheable]
#[derive(Debug, Default)]
struct CssModuleExports {
  #[cacheable(with=AsMap<AsPreset, AsVec>)]
  javascript: FxHashMap<SmolStr, Arc<[CssExportPart]>>,
  #[cacheable(with=AsMap)]
  symbols: FxHashMap<DependencyId, Option<String>>,
  concat_targets: Option<CssExportTargets>,
}

#[cacheable]
#[derive(Debug, Clone, Copy)]
pub(crate) struct CssExportTarget {
  pub module: ModuleIdentifier,
  pub supports_javascript: bool,
}

#[cacheable]
#[derive(Debug, Default)]
struct CssExportTargets {
  #[cacheable(with=AsMap)]
  by_dependency: FxHashMap<DependencyId, CssExportTarget>,
  #[cacheable(with=AsMap<AsPreset>)]
  by_request: FxHashMap<SmolStr, CssExportTarget>,
}

impl CssExportTargets {
  fn new(compilation: &Compilation, module: &dyn Module) -> Self {
    let module_graph = compilation.get_module_graph();
    let mut targets = Self::default();
    for dependency in module.get_dependencies() {
      let Some(target) = module_graph.get_module_by_dependency_id(dependency.id()) else {
        continue;
      };
      let target = CssExportTarget {
        module: target.identifier(),
        supports_javascript: target
          .source_types(module_graph)
          .contains(&SourceType::JavaScript),
      };
      targets.by_dependency.insert(*dependency.id(), target);
      if let Some(request) = dependency_request(dependency.as_ref()) {
        // The ordinary request fallback uses the first connected dependency.
        targets.by_request.entry(request.into()).or_insert(target);
      }
    }
    targets
  }

  fn get(&self, request: &str, id: Option<&DependencyId>) -> Option<CssExportTarget> {
    id.and_then(|id| self.by_dependency.get(id))
      .or_else(|| self.by_request.get(request))
      .copied()
  }
}

fn cached_css_export_target<'a>(
  compilation: &'a Compilation,
  targets: &mut IdentifierMap<CssExportTargets>,
  module: &dyn Module,
  request: &str,
  id: Option<&DependencyId>,
) -> Option<&'a dyn Module> {
  let target = targets
    .entry(module.identifier())
    .or_insert_with(|| CssExportTargets::new(compilation, module))
    .get(request, id)?;
  compilation
    .get_module_graph()
    .module_by_identifier(&target.module)
    .map(|module| module.as_ref())
}

#[cacheable_dyn]
impl CodeGenerationDataItem for CodeGenerationDataCssExports {}

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

fn find_css_export_target<'a>(
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
  cyclic: bool,
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
      cyclic: false,
    })
  }
}

fn resolve_css_exports<'a, T: Clone>(
  compilation: &'a Compilation,
  targets: &mut IdentifierMap<CssExportTargets>,
  roots: impl IntoIterator<Item = (&'a dyn Module, SmolStr)>,
  should_follow: impl Fn(ModuleIdentifier) -> bool,
  leaf: impl Fn(&dyn Module, &CssExportPart) -> Option<T>,
) -> IdentifierMap<FxHashMap<SmolStr, Vec<T>>> {
  let mut values = IdentifierMap::<FxHashMap<SmolStr, Vec<T>>>::default();
  let mut memo = FxHashMap::<(ModuleIdentifier, SmolStr), Vec<T>>::default();
  for (module, name) in roots {
    let exports = values.entry(module.identifier()).or_default();
    if exports.contains_key(&name) {
      continue;
    }
    if let Some(resolved) = memo.get(&(module.identifier(), name.clone())) {
      exports.insert(name, resolved.clone());
      continue;
    }
    let Some(root) = CssExportFrame::new(module, &name) else {
      exports.insert(name, Vec::new());
      continue;
    };
    let mut active = FxHashSet::from_iter([(module.identifier(), name.clone())]);
    let mut stack = vec![root];
    while let Some(frame) = stack.last_mut() {
      let Some(export) = frame.exports.get_index(frame.next_index) else {
        let frame = stack.pop().expect("CSS export frame should exist");
        let key = (frame.module.identifier(), frame.name);
        active.remove(&key);
        // A cyclic result depends on the active ancestors. Only acyclic
        // branches can be reused while resolving other exports.
        if !frame.cyclic {
          memo.insert(key, frame.resolved.clone());
        }
        if let Some(parent) = stack.last_mut() {
          parent.current.extend(frame.resolved);
          parent.cyclic |= frame.cyclic;
        } else {
          exports.insert(name.clone(), frame.resolved);
        }
        continue;
      };
      // Compositions add a space between values, while the parts of a compound
      // ICSS value concatenate verbatim (including punctuation and whitespace).
      let Some(export) = export.parts.get(frame.next_part) else {
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
      let target = export.from.as_deref().and_then(|request| {
        cached_css_export_target(
          compilation,
          targets,
          frame.module,
          request,
          export.id.as_ref(),
        )
      });
      if let Some(target) = target.filter(|target| should_follow(target.identifier())) {
        let key = (target.identifier(), export.ident.clone());
        // Cut cycles on the active path, retaining repeated leaves in
        // independent branches and their original order.
        if active.contains(&key) {
          frame.cyclic = true;
        } else if let Some(resolved) = memo.get(&key) {
          frame.current.extend_from_slice(resolved);
        } else if let Some(child) = CssExportFrame::new(target, &export.ident) {
          active.insert(key);
          stack.push(child);
        }
      } else if let Some(value) = leaf(frame.module, export) {
        frame.current.push(value);
      }
    }
  }
  values
}

pub(crate) fn prepare_css_exports(
  compilation: &Compilation,
  module: &dyn Module,
  data: &mut CodeGenerationData,
) {
  if !data.contains::<CodeGenerationDataCssExports>() {
    data.insert(CodeGenerationDataCssExports::default());
  }
  let data = data
    .get_mut::<CodeGenerationDataCssExports>()
    .expect("CSS export resolutions should be initialized");
  let identifier = module.identifier();
  if data.modules.contains_key(&identifier) {
    return;
  }

  let roots = module
    .build_info()
    .css
    .as_deref()
    .map(|css| {
      css
        .exports
        .keys()
        .map(|name| (module, name.clone()))
        .collect::<Vec<_>>()
    })
    .unwrap_or_default();
  let mut javascript = resolve_css_exports(
    compilation,
    &mut data.targets,
    roots,
    |target| target == identifier,
    |_, part| Some(part.clone()),
  )
  .remove(&identifier)
  .unwrap_or_default();
  // A pure ICSS cycle has no concrete value. Preserve its runtime resolution.
  if let Some(css) = module.build_info().css.as_deref() {
    for (name, parts) in &mut javascript {
      if parts.is_empty() {
        for export in css.exports.get(name).expect("CSS export should exist") {
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
    }
  }

  let symbols = module
    .get_dependencies()
    .iter()
    .filter_map(|dependency| dependency.downcast_ref::<CssIcssSymbolDependency>())
    .collect::<Vec<_>>();
  let mut static_roots = Vec::new();
  for dependency in &symbols {
    for part in &dependency.value().parts {
      if let Some(request) = &part.from
        && let Some(target) = cached_css_export_target(
          compilation,
          &mut data.targets,
          module,
          request,
          part.id.as_ref(),
        )
      {
        static_roots.push((target, part.ident.clone()));
      }
    }
  }
  prepare_static_css_exports(compilation, data, static_roots);
  let symbols = symbols
    .into_iter()
    .map(|dependency| {
      let resolved = dependency
        .value()
        .parts
        .iter()
        .map(|part| {
          if let Some(request) = &part.from {
            let target = data
              .targets
              .get(&identifier)?
              .get(request, part.id.as_ref())?;
            data
              .values
              .get(&target.module)?
              .get(&part.ident)?
              .as_deref()
          } else {
            Some(part.ident.as_str())
          }
        })
        .collect::<Option<String>>();
      (*dependency.id(), resolved)
    })
    .collect();
  data.modules.insert(
    identifier,
    CssModuleExports {
      javascript: javascript
        .into_iter()
        .map(|(name, parts)| (name, Arc::from(parts)))
        .collect(),
      symbols,
      concat_targets: None,
    },
  );
}

fn prepare_static_css_exports<'a>(
  compilation: &'a Compilation,
  data: &mut CodeGenerationDataCssExports,
  roots: impl IntoIterator<Item = (&'a dyn Module, SmolStr)>,
) {
  let values = resolve_css_exports(
    compilation,
    &mut data.targets,
    roots.into_iter().filter(|(module, name)| {
      !data
        .values
        .get(&module.identifier())
        .is_some_and(|exports| exports.contains_key(name))
    }),
    |_| true,
    |module, export| {
      export
        .from
        .is_none()
        .then(|| replace_css_module_id_placeholder(&export.ident, compilation, module).into_owned())
    },
  );
  for (identifier, exports) in values {
    data.values.entry(identifier).or_default().extend(
      exports
        .into_iter()
        .map(|(name, values)| (name, (!values.is_empty()).then(|| values.concat()))),
    );
  }
}

pub(crate) fn prepare_css_concat_exports(
  compilation: &Compilation,
  module: &dyn Module,
  data: &mut CodeGenerationData,
) {
  let data = data
    .get_mut::<CodeGenerationDataCssExports>()
    .expect("CSS export resolutions should be initialized");
  let exports = data
    .modules
    .get(&module.identifier())
    .expect("CSS module exports should be prepared");
  if exports.concat_targets.is_some() {
    return;
  }
  let targets = data
    .targets
    .entry(module.identifier())
    .or_insert_with(|| CssExportTargets::new(compilation, module));
  let targets = prepare_concat_targets(compilation, module, targets);
  let mut roots = Vec::new();
  if let Some(css) = module.build_info().css.as_deref() {
    for export in css
      .exports
      .values()
      .flatten()
      .flat_map(|export| &export.parts)
    {
      if let Some(request) = &export.from
        && let Some(target) = targets.get(request, export.id.as_ref())
        && !target.supports_javascript
        && let Some(module) = compilation
          .get_module_graph()
          .module_by_identifier(&target.module)
      {
        roots.push((module.as_ref(), export.ident.clone()));
      }
    }
  }
  prepare_static_css_exports(compilation, data, roots);
  data
    .modules
    .get_mut(&module.identifier())
    .expect("CSS module exports should be prepared")
    .concat_targets = Some(targets);
}

fn prepare_concat_targets(
  compilation: &Compilation,
  module: &dyn Module,
  direct_targets: &CssExportTargets,
) -> CssExportTargets {
  let module_graph = compilation.get_module_graph();
  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
  let identifier = module.identifier();
  let current_chunks = (chunk_graph.get_number_of_module_chunks(identifier) > 0)
    .then(|| chunk_graph.get_module_chunks(identifier));
  let priority = |target: CssExportTarget| {
    let shares_chunk = current_chunks.is_some_and(|current| {
      chunk_graph.get_number_of_module_chunks(target.module) > 0
        && chunk_graph
          .get_module_chunks(target.module)
          .iter()
          .any(|chunk| current.contains(chunk))
    });
    (
      target.supports_javascript,
      shares_chunk,
      ChunkGraph::get_module_id(&compilation.module_ids_artifact, target.module).is_some(),
    )
  };
  let mut targets = CssExportTargets::default();
  let mut requests = FxHashSet::default();
  if let Some(css) = module.build_info().css.as_deref() {
    for export in css
      .exports
      .values()
      .flatten()
      .flat_map(|export| &export.parts)
    {
      let Some(request) = &export.from else {
        continue;
      };
      if let Some(id) = export.id
        && let Some(target) = direct_targets.by_dependency.get(&id)
      {
        targets.by_dependency.insert(id, *target);
      } else {
        requests.insert(request.clone());
      }
    }
  }
  // Concatenation picks the highest-priority original dependency before
  // selecting an alternative JS module. Preserve the last dependency on ties.
  for dependency in module.get_dependencies() {
    if let Some(request) = dependency_request(dependency.as_ref())
      && requests.contains(request)
      && let Some(target) = direct_targets.by_dependency.get(dependency.id())
    {
      let current = targets.by_request.entry(request.into()).or_insert(*target);
      if priority(*target) >= priority(*current) {
        *current = *target;
      }
    }
  }
  let mut alternatives = None;
  let mut replacements = IdentifierMap::default();
  for target in targets
    .by_dependency
    .values_mut()
    .chain(targets.by_request.values_mut())
  {
    if target.supports_javascript {
      continue;
    }
    *target = *replacements.entry(target.module).or_insert_with(|| {
      // Build the candidate index at most once for this issuer, and only when
      // a CSS-only target needs an alternative. Exports then reuse the result.
      let alternatives = alternatives.get_or_insert_with(|| {
        let mut candidates = FxHashMap::<Option<Box<str>>, CssExportTarget>::default();
        for (_, module) in module_graph.modules() {
          if !module
            .source_types(module_graph)
            .contains(&SourceType::JavaScript)
          {
            continue;
          }
          let candidate = CssExportTarget {
            module: module.identifier(),
            supports_javascript: true,
          };
          let current = candidates
            .entry(module.name_for_condition())
            .or_insert(candidate);
          if priority(candidate) >= priority(*current) {
            *current = candidate;
          }
        }
        candidates
      });
      let module = module_graph
        .module_by_identifier(&target.module)
        .expect("CSS export target should exist");
      alternatives
        .get(&module.name_for_condition())
        .copied()
        .unwrap_or(*target)
    });
  }
  targets
}

pub(crate) fn get_css_export_target(
  data: &CodeGenerationData,
  module: ModuleIdentifier,
  request: &str,
  id: Option<&DependencyId>,
  concatenated: bool,
) -> Option<CssExportTarget> {
  let data = data.get::<CodeGenerationDataCssExports>()?;
  let targets = if concatenated {
    data.modules.get(&module)?.concat_targets.as_ref()?
  } else {
    data.targets.get(&module)?
  };
  targets.get(request, id)
}

pub(crate) fn get_css_exports(
  data: &CodeGenerationData,
  module: ModuleIdentifier,
  name: &str,
) -> Arc<[CssExportPart]> {
  Arc::clone(
    data
      .get::<CodeGenerationDataCssExports>()
      .and_then(|data| data.modules.get(&module))
      .and_then(|module| module.javascript.get(name))
      .expect("CSS exports should be prepared"),
  )
}

pub(crate) fn get_css_export<'a>(
  data: &'a CodeGenerationData,
  module: ModuleIdentifier,
  name: &str,
) -> Option<&'a str> {
  data
    .get::<CodeGenerationDataCssExports>()?
    .values
    .get(&module)?
    .get(name)?
    .as_deref()
}

pub(crate) fn get_icss_symbol<'a>(
  data: &'a CodeGenerationData,
  module: ModuleIdentifier,
  dependency: &DependencyId,
) -> Option<&'a str> {
  data
    .get::<CodeGenerationDataCssExports>()?
    .modules
    .get(&module)?
    .symbols
    .get(dependency)?
    .as_deref()
}
