use std::sync::Arc;

use rayon::prelude::*;
use rspack_collections::{IdentifierIndexMap, IdentifierMap};
use rspack_error::{Error, Result};
use rspack_util::{
  SpanExt,
  atom::Atom,
  fx_hash::{FxHashMap, FxHashSet},
  itoa,
};
use swc_core::common::SyntaxContext;
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::{EsVersion, Program};
use swc_experimental_ecma_parser::{EsSyntax, Parser, StringSource, Syntax};
use swc_experimental_ecma_semantic::resolver::resolver;

use crate::{
  BuildMetaDefaultObject, BuildMetaExportsType, Compilation, ConcatenatedModuleInfo, Context,
  ExportInfo, ExportProvided, ExportsInfoArtifact, ExportsType, FindTargetResult, ModuleGraph,
  ModuleGraphCacheArtifact, ModuleIdentifier, ModuleInfo, ModuleStaticCache, RuntimeSpec, UsedName,
  collect_ident, escape_name_atom_ref, find_target, get_cached_readable_identifier,
  split_readable_identifier, to_identifier_with_escaped,
};

/// Reusable read-only context shared by concatenation implementations.
pub struct ConcatenationContext<'a> {
  pub module_graph: &'a ModuleGraph,
  pub module_graph_cache: &'a ModuleGraphCacheArtifact,
  pub module_static_cache: &'a ModuleStaticCache,
  pub exports_info_artifact: &'a ExportsInfoArtifact,
  pub runtime: Option<&'a RuntimeSpec>,
  pub compiler_context: &'a Context,
  pub escaped_names: FxHashMap<Atom, Atom>,
  pub escaped_identifiers: FxHashMap<String, Vec<Atom>>,
  pub module_identifiers: IdentifierMap<Vec<Atom>>,
}

/// Allocates unique JavaScript identifiers while retaining per-base suffix cursors.
#[derive(Debug, Clone, Default)]
pub struct ConcatenationNameAllocator {
  used_names: FxHashSet<Atom>,
  used_strings: FxHashSet<String>,
  suffix_counters: FxHashMap<Atom, u32>,
}

impl ConcatenationNameAllocator {
  pub fn new(used_names: FxHashSet<Atom>) -> Self {
    let mut used_strings = FxHashSet::default();
    used_strings.reserve(used_names.len());
    for name in &used_names {
      used_strings.insert(name.as_ref().to_string());
    }

    Self {
      used_names,
      used_strings,
      suffix_counters: FxHashMap::default(),
    }
  }

  pub fn contains(&self, name: &Atom) -> bool {
    self.used_names.contains(name)
  }

  pub fn insert(&mut self, name: Atom) {
    self.used_strings.insert(name.as_ref().to_string());
    self.used_names.insert(name);
  }

  pub fn extend(&mut self, names: impl IntoIterator<Item = Atom>) {
    for name in names {
      self.insert(name);
    }
  }

  pub fn merge(&mut self, other: Self) {
    self.used_names.extend(other.used_names);
    self.used_strings.extend(other.used_strings);
    for (base, next_suffix) in other.suffix_counters {
      self
        .suffix_counters
        .entry(base)
        .and_modify(|current| *current = (*current).max(next_suffix))
        .or_insert(next_suffix);
    }
  }

  /// Returns a unique name and reserves it in this allocator.
  pub fn find_new_name(&mut self, old_name: &str, extra_info: &[Atom]) -> Atom {
    let mut name = old_name.to_string();

    for info_part in extra_info {
      let info_str = info_part.as_ref();
      let mut new_name = String::with_capacity(info_str.len() + 1 + name.len());
      new_name.push_str(info_str);
      if !name.is_empty() {
        if name.starts_with('_') || info_str.ends_with('_') {
          new_name.push_str(&name);
        } else {
          new_name.push('_');
          new_name.push_str(&name);
        }
      }
      name = new_name;

      let escaped = to_identifier_with_escaped(name.clone());
      if !self.used_strings.contains(&escaped) {
        self.used_strings.insert(escaped.clone());
        let candidate: Atom = escaped.into();
        self.used_names.insert(candidate.clone());
        return candidate;
      }
    }

    let base_str = to_identifier_with_escaped(name);
    if !base_str.is_empty() && !self.used_strings.contains(&base_str) {
      self.used_strings.insert(base_str.clone());
      let base: Atom = base_str.into();
      self.used_names.insert(base.clone());
      return base;
    }

    let base: Atom = base_str.into();
    let counter = self.suffix_counters.entry(base.clone()).or_insert(0);
    let mut i = *counter;
    let mut i_buffer = itoa::Buffer::new();

    let mut base_with_underscore = String::with_capacity(base.len() + 1);
    base_with_underscore.push_str(base.as_ref());
    base_with_underscore.push('_');

    let mut numbered = String::with_capacity(base_with_underscore.len() + 8);
    loop {
      numbered.clear();
      numbered.push_str(&base_with_underscore);
      numbered.push_str(i_buffer.format(i));

      if !self.used_strings.contains(&numbered) {
        self.used_strings.insert(numbered.clone());
        let candidate: Atom = Atom::from(numbered.as_str());
        self.used_names.insert(candidate.clone());
        *counter = i + 1;
        return candidate;
      }

      i += 1;
    }
  }

  pub fn find_new_module_name(
    &mut self,
    old_name: &str,
    module: &ModuleIdentifier,
    context: &ConcatenationContext,
  ) -> Atom {
    self.find_new_name(old_name, context.module_identifier(module))
  }

  pub fn find_new_binding_name(
    &mut self,
    name: &Atom,
    extra_info: &[Atom],
    context: &ConcatenationContext,
  ) -> Atom {
    self.find_new_name(
      context
        .escaped_names
        .get(name)
        .expect("should have escaped name")
        .as_ref(),
      extra_info,
    )
  }
}

pub enum ConcatenationInterop {
  Namespace,
  Namespace2,
}

pub struct ConcatenationBindingPlan {
  pub info_id: ModuleIdentifier,
  pub export_name: Vec<Atom>,
  /// The defer flag on the re-export edge closest to the resolved target.
  pub reexport_deferred: Option<bool>,
  pub target: ConcatenationBindingTarget,
}

pub enum ConcatenationBindingTarget {
  InteropNamespace(ConcatenationInterop),
  InteropDefault,
  EsModule(Vec<Atom>),
  UnsupportedDefaultImport,
  Namespace,
  Circular,
  Direct {
    symbol: Atom,
    used_name: Option<UsedName>,
  },
  Raw(Atom),
  Inlined(UsedName),
  NamespaceExport(UsedName),
  Missing(Option<UsedName>),
  External(Option<UsedName>),
}

/// Stateful backend-independent binding resolver.
///
/// Shared read-only dependencies live in `context`, module binding state lives
/// in `module_to_info_map`, and only recursion-local cycle detection is created
/// per `resolve` call.
pub struct ConcatenationBindingResolver<'a> {
  pub context: &'a ConcatenationContext<'a>,
  pub module_to_info_map: &'a mut IdentifierIndexMap<ModuleInfo>,
  pub normalize_export_name: Option<fn(&ModuleGraph, &ModuleIdentifier, &mut Vec<Atom>)>,
}

impl<'a> ConcatenationBindingResolver<'a> {
  pub fn resolve(
    &self,
    info_id: &ModuleIdentifier,
    export_name: Vec<Atom>,
    strict_esm_module: bool,
  ) -> ConcatenationBindingPlan {
    self.resolve_inner(
      info_id,
      export_name,
      strict_esm_module,
      &mut Default::default(),
    )
  }

  fn resolve_inner(
    &self,
    info_id: &ModuleIdentifier,
    mut export_name: Vec<Atom>,
    strict_esm_module: bool,
    visited_exports: &mut FxHashSet<ExportInfo>,
  ) -> ConcatenationBindingPlan {
    let module_graph = self.context.module_graph;
    let module_graph_cache = self.context.module_graph_cache;
    let exports_info_artifact = self.context.exports_info_artifact;
    let module_to_info_map = &*self.module_to_info_map;
    let runtime = self.context.runtime;
    let info = module_to_info_map
      .get(info_id)
      .expect("should have module info");
    let module = module_graph
      .module_by_identifier(&info.id())
      .expect("should have module");
    let exports_type = module.get_exports_type(
      module_graph,
      module_graph_cache,
      exports_info_artifact,
      strict_esm_module,
    );

    if let Some(normalize_export_name) = self.normalize_export_name {
      normalize_export_name(module_graph, info_id, &mut export_name);
    }

    if export_name.is_empty() {
      match exports_type {
        ExportsType::DefaultOnly => {
          return ConcatenationBindingPlan {
            info_id: *info_id,
            export_name,
            reexport_deferred: None,
            target: ConcatenationBindingTarget::InteropNamespace(ConcatenationInterop::Namespace2),
          };
        }
        ExportsType::DefaultWithNamed => {
          return ConcatenationBindingPlan {
            info_id: *info_id,
            export_name,
            reexport_deferred: None,
            target: ConcatenationBindingTarget::InteropNamespace(ConcatenationInterop::Namespace),
          };
        }
        _ => {}
      }
    } else {
      match exports_type {
        ExportsType::Namespace => {}
        ExportsType::DefaultWithNamed => match export_name.first().map(Atom::as_str) {
          Some("default") => export_name = export_name[1..].to_vec(),
          Some("__esModule") => {
            return ConcatenationBindingPlan {
              info_id: *info_id,
              export_name: export_name.clone(),
              reexport_deferred: None,
              target: ConcatenationBindingTarget::EsModule(export_name[1..].to_vec()),
            };
          }
          _ => {}
        },
        ExportsType::DefaultOnly => {
          if export_name.first().map(Atom::as_str) == Some("__esModule") {
            return ConcatenationBindingPlan {
              info_id: *info_id,
              export_name: export_name.clone(),
              reexport_deferred: None,
              target: ConcatenationBindingTarget::EsModule(export_name[1..].to_vec()),
            };
          }

          let first_export = export_name.remove(0);
          if first_export != "default" {
            return ConcatenationBindingPlan {
              info_id: *info_id,
              export_name,
              reexport_deferred: None,
              target: ConcatenationBindingTarget::UnsupportedDefaultImport,
            };
          }
        }
        ExportsType::Dynamic => match export_name.first().map(Atom::as_str) {
          Some("default") => {
            return ConcatenationBindingPlan {
              info_id: *info_id,
              export_name: export_name[1..].to_vec(),
              reexport_deferred: None,
              target: ConcatenationBindingTarget::InteropDefault,
            };
          }
          Some("__esModule") => {
            return ConcatenationBindingPlan {
              info_id: *info_id,
              export_name: export_name.clone(),
              reexport_deferred: None,
              target: ConcatenationBindingTarget::EsModule(export_name[1..].to_vec()),
            };
          }
          _ => {}
        },
      }
    }

    if export_name.is_empty() {
      return ConcatenationBindingPlan {
        info_id: info.id(),
        export_name,
        reexport_deferred: None,
        target: ConcatenationBindingTarget::Namespace,
      };
    }

    let exports_info = exports_info_artifact.get_exports_info_data(&info.id());
    let export_info = exports_info.get_export_info_without_mut_module_graph(&export_name[0]);
    let export_info_id = export_info.id();

    if !visited_exports.insert(export_info_id) {
      return ConcatenationBindingPlan {
        info_id: info.id(),
        export_name,
        reexport_deferred: None,
        target: ConcatenationBindingTarget::Circular,
      };
    }

    match info {
      ModuleInfo::Concatenated(info) => {
        if matches!(export_info.provided(), Some(ExportProvided::NotProvided)) {
          return ConcatenationBindingPlan {
            info_id: info.module,
            export_name,
            reexport_deferred: None,
            target: ConcatenationBindingTarget::Namespace,
          };
        }

        let export_id = export_name.first().cloned();
        let used_name = exports_info.get_used_name(exports_info_artifact, runtime, &export_name);

        if let Some(export_id) = &export_id
          && let Some(direct_export) = info.export_map.as_ref().and_then(|map| map.get(export_id))
        {
          return ConcatenationBindingPlan {
            info_id: info.module,
            export_name,
            reexport_deferred: None,
            target: ConcatenationBindingTarget::Direct {
              symbol: direct_export.as_str().into(),
              used_name,
            },
          };
        }

        if let Some(export_id) = &export_id
          && let Some(raw_export) = info
            .raw_export_map
            .as_ref()
            .and_then(|map| map.get(export_id))
        {
          return ConcatenationBindingPlan {
            info_id: info.module,
            export_name,
            reexport_deferred: None,
            target: ConcatenationBindingTarget::Raw(raw_export.as_str().into()),
          };
        }

        match find_target(
          &export_info,
          module_graph,
          exports_info_artifact,
          Arc::new(|module: &ModuleIdentifier| module_to_info_map.contains_key(module)),
          &mut Default::default(),
        ) {
          FindTargetResult::NoTarget => {}
          FindTargetResult::InvalidTarget(target) => {
            if let Some(export) = target.export {
              let target_exports_info = exports_info_artifact.get_exports_info_data(&target.module);
              if let Some(used_name @ UsedName::Inlined(_)) =
                target_exports_info.get_used_name(exports_info_artifact, runtime, &export)
              {
                return ConcatenationBindingPlan {
                  info_id: info.module,
                  export_name,
                  reexport_deferred: None,
                  target: ConcatenationBindingTarget::Inlined(used_name),
                };
              }
            }
            panic!(
              "Target module of reexport is not part of the concatenation (export '{export_id:?}')"
            );
          }
          FindTargetResult::ValidTarget(reexport) => {
            if let Some(ref_info) = module_to_info_map.get(&reexport.module) {
              let target_export_name = if let Some(reexport_export) = reexport.export {
                [reexport_export, export_name[1..].to_vec()].concat()
              } else {
                export_name[1..].to_vec()
              };
              let mut plan = self.resolve_inner(
                &ref_info.id(),
                target_export_name,
                module.build_meta().strict_esm_module(),
                visited_exports,
              );
              plan.reexport_deferred.get_or_insert(reexport.defer);
              return plan;
            }
          }
        }

        if info.namespace_export_symbol.is_some() {
          return ConcatenationBindingPlan {
            info_id: info.module,
            export_name,
            reexport_deferred: None,
            target: ConcatenationBindingTarget::NamespaceExport(
              used_name.expect("should have export name"),
            ),
          };
        }

        ConcatenationBindingPlan {
          info_id: info.module,
          export_name,
          reexport_deferred: None,
          target: ConcatenationBindingTarget::Missing(used_name),
        }
      }
      ModuleInfo::External(info) => {
        let used_name = exports_info.get_used_name(exports_info_artifact, runtime, &export_name);
        ConcatenationBindingPlan {
          info_id: info.module,
          export_name,
          reexport_deferred: None,
          target: ConcatenationBindingTarget::External(used_name),
        }
      }
    }
  }
}

impl ConcatenationNameAllocator {
  pub fn assign_module_binding_names(
    &mut self,
    module_info: &mut ConcatenatedModuleInfo,
    context: &ConcatenationContext,
  ) {
    let escaped_identifier = context.module_identifier(&module_info.module);
    for (name, ctxt) in module_info.binding_to_ref.keys() {
      if ctxt != &module_info.module_ctxt {
        continue;
      }

      let internal_name = if self.contains(name) {
        self.find_new_name(
          context
            .escaped_names
            .get(name)
            .expect("should have escaped name")
            .as_ref(),
          escaped_identifier,
        )
      } else {
        self.insert(name.clone());
        name.clone()
      };
      module_info
        .internal_names
        .insert(name.clone(), internal_name);
    }
  }

  pub fn assign_import_binding_name(
    &mut self,
    imported_name: &Atom,
    existing_name: Option<&Atom>,
    source: &str,
    module_info: &mut ConcatenatedModuleInfo,
    context: &ConcatenationContext,
  ) -> Atom {
    let should_update_raw_export = existing_name.is_some() || self.contains(imported_name);
    let internal_name = existing_name.cloned().unwrap_or_else(|| {
      if !self.contains(imported_name) {
        self.insert(imported_name.clone());
        return imported_name.clone();
      }

      if imported_name == "default" {
        self.find_new_name("", context.source_identifier(source))
      } else {
        self.find_new_name(
          context
            .escaped_names
            .get(imported_name)
            .expect("should have escaped name")
            .as_ref(),
          context.module_identifier(&module_info.module),
        )
      }
    });

    if should_update_raw_export
      && let Some(raw_export_map) = module_info.raw_export_map.as_mut()
      && raw_export_map.contains_key(imported_name)
    {
      raw_export_map.insert(imported_name.clone(), internal_name.to_string());
    }
    module_info
      .internal_names
      .insert(imported_name.clone(), internal_name.clone());
    internal_name
  }

  pub fn assign_interop_names(
    &mut self,
    module_info: &mut ModuleInfo,
    context: &ConcatenationContext,
  ) {
    let module = module_info.id();
    let build_meta = context
      .module_graph
      .module_by_identifier(&module)
      .expect("should have module")
      .build_meta();
    let exports_type: BuildMetaExportsType = build_meta.exports_type();
    let default_object: BuildMetaDefaultObject = build_meta.default_object();
    if exports_type != BuildMetaExportsType::Namespace {
      module_info.set_interop_namespace_object_name(Some(self.find_new_module_name(
        "namespaceObject",
        &module,
        context,
      )));
    }

    if exports_type == BuildMetaExportsType::Default
      && !matches!(default_object, BuildMetaDefaultObject::Redirect)
    {
      module_info.set_interop_namespace_object2_name(Some(self.find_new_module_name(
        "namespaceObject2",
        &module,
        context,
      )));
    }

    if matches!(
      exports_type,
      BuildMetaExportsType::Dynamic | BuildMetaExportsType::Unset
    ) {
      module_info.set_interop_default_access_name(Some(
        self.find_new_module_name("default", &module, context),
      ));
    }
  }
}

pub fn analyze_module_scope(
  source: &str,
  jsx: bool,
  module_info: &mut ConcatenatedModuleInfo,
) -> Result<()> {
  let allocator = Allocator::new();
  let lexer = swc_experimental_ecma_parser::Lexer::new(
    &allocator,
    Syntax::Es(EsSyntax {
      jsx,
      ..Default::default()
    }),
    EsVersion::EsNext,
    StringSource::new(source),
    None,
  );
  let mut parser = Parser::new_from(&allocator, lexer);
  let parsed_module = parser.parse_module().map_err(|error| {
    Error::from_string(
      Some(source.to_owned()),
      error.span().real_lo() as usize,
      error.span().real_hi() as usize,
      "JavaScript parse error:\n".to_string(),
      error.kind().msg().to_string(),
    )
  })?;
  let program = Program::Module(allocator.boxed(parsed_module));
  let semantic = resolver(&program);
  let identifiers = collect_ident(&allocator, &program);

  module_info.module_ctxt = SyntaxContext::from_u32(semantic.top_level_scope_id().raw());
  module_info.global_ctxt = SyntaxContext::from_u32(semantic.unresolved_scope_id().raw());

  let top_level_scope_id = semantic.top_level_scope_id();
  module_info.all_used_names.clear();
  module_info.binding_to_ref.clear();
  module_info.all_used_names.reserve(identifiers.len());
  module_info.idents.reserve(identifiers.len());
  module_info.global_scope_ident.reserve(identifiers.len());
  module_info.binding_to_ref.reserve(identifiers.len());

  for identifier in identifiers {
    let scope = semantic.node_scope(&identifier.id);
    let is_global = SyntaxContext::from_u32(scope.raw()) == module_info.global_ctxt;
    let legacy = if is_global {
      let legacy = identifier.to_legacy(&semantic);
      module_info.global_scope_ident.push(legacy.clone());
      module_info.all_used_names.insert(legacy.id.sym.clone());
      Some(legacy)
    } else {
      None
    };

    if identifier.is_class_expr_with_ident {
      module_info
        .all_used_names
        .insert(Atom::from(identifier.id.sym.as_str()));
      continue;
    }

    if scope != top_level_scope_id {
      module_info
        .all_used_names
        .insert(Atom::from(identifier.id.sym.as_str()));
    }

    let legacy = legacy.unwrap_or_else(|| identifier.to_legacy(&semantic));
    module_info.idents.push(legacy.clone());
    module_info
      .binding_to_ref
      .entry((legacy.id.sym.clone(), legacy.id.ctxt))
      .or_default()
      .push(legacy);
  }

  module_info.has_ast = true;
  Ok(())
}

impl<'a> ConcatenationContext<'a> {
  pub fn prepare(
    module_to_info_map: &IdentifierIndexMap<ModuleInfo>,
    compilation: &'a Compilation,
    runtime: Option<&'a RuntimeSpec>,
  ) -> ConcatenationContext<'a> {
    let module_graph = compilation.get_module_graph();
    let module_graph_cache = &compilation.module_graph_cache_artifact;
    let module_static_cache = &compilation.module_static_cache;
    let exports_info_artifact = &compilation.exports_info_artifact;
    let compiler_context = &compilation.options.context;
    let (escaped_name_entries, escaped_identifier_entries) = module_to_info_map
      .par_values()
      .map(|info| {
        let (name_capacity, identifier_capacity) = match info {
          ModuleInfo::Concatenated(info) => {
            let import_map = info.import_map.as_ref();
            let import_sources = import_map.map_or(0, |map| map.len());
            let imported_names = import_map.map_or(0, |map| {
              map
                .values()
                .map(|imported| {
                  imported.specifiers.len() + usize::from(imported.namespace.is_some())
                })
                .sum::<usize>()
            });
            (
              info.binding_to_ref.len() + imported_names,
              1 + import_sources,
            )
          }
          ModuleInfo::External(_) => (0, 1),
        };
        let mut escaped_names =
          FxHashMap::with_capacity_and_hasher(name_capacity, Default::default());
        let mut escaped_identifiers = Vec::with_capacity(identifier_capacity);
        let readable_identifier = get_cached_readable_identifier(
          &info.id(),
          module_graph,
          module_static_cache,
          compiler_context,
        );
        let module_identifier = split_readable_identifier(&readable_identifier);
        escaped_identifiers.push((Some(info.id()), readable_identifier, module_identifier));

        if let ModuleInfo::Concatenated(info) = info {
          for (identifier, _) in &info.binding_to_ref {
            escaped_names
              .entry(identifier.0.clone())
              .or_insert_with(|| escape_name_atom_ref(&identifier.0));
          }

          if let Some(import_map) = &info.import_map {
            for ((source, _), imported) in import_map {
              escaped_identifiers.push((
                None,
                source.clone(),
                split_readable_identifier(source.as_str()),
              ));
              for atom in &imported.specifiers {
                escaped_names
                  .entry(atom.clone())
                  .or_insert_with(|| escape_name_atom_ref(atom));
              }
              if let Some(namespace) = &imported.namespace {
                escaped_names
                  .entry(namespace.clone())
                  .or_insert_with(|| escape_name_atom_ref(namespace));
              }
            }
          }
        }

        (
          escaped_names.into_iter().collect::<Vec<_>>(),
          escaped_identifiers,
        )
      })
      .reduce(
        || (Vec::new(), Vec::new()),
        |mut left, mut right| {
          left.0.append(&mut right.0);
          left.1.append(&mut right.1);
          left
        },
      );

    let mut escaped_names =
      FxHashMap::with_capacity_and_hasher(escaped_name_entries.len(), Default::default());
    escaped_names.extend(escaped_name_entries);
    let mut escaped_identifiers =
      FxHashMap::with_capacity_and_hasher(escaped_identifier_entries.len(), Default::default());
    let mut module_identifiers = IdentifierMap::default();
    module_identifiers.reserve(module_to_info_map.len());
    for (module, identifier, escaped_identifier) in escaped_identifier_entries {
      if let Some(module) = module {
        module_identifiers.insert(module, escaped_identifier.clone());
      }
      escaped_identifiers.insert(identifier, escaped_identifier);
    }

    Self {
      module_graph,
      module_graph_cache,
      module_static_cache,
      exports_info_artifact,
      runtime,
      compiler_context,
      escaped_names,
      escaped_identifiers,
      module_identifiers,
    }
  }

  pub fn readable_identifier(&self, module: &ModuleIdentifier) -> String {
    get_cached_readable_identifier(
      module,
      self.module_graph,
      self.module_static_cache,
      self.compiler_context,
    )
  }

  pub fn module_identifier(&self, module: &ModuleIdentifier) -> &[Atom] {
    self
      .module_identifiers
      .get(module)
      .expect("should have escaped identifier")
  }

  pub fn source_identifier(&self, source: &str) -> &[Atom] {
    self
      .escaped_identifiers
      .get(source)
      .expect("should have escaped identifier")
  }
}
