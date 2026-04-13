use std::{
  collections::{self},
  hash::BuildHasher,
  sync::{Arc, LazyLock},
};

use rayon::{iter::Either, prelude::*};
use rspack_collections::{IdentifierIndexMap, IdentifierIndexSet, IdentifierMap};
use rspack_core::{
  BuildMetaDefaultObject, BuildMetaExportsType, ChunkGraph, ChunkInitFragments, ChunkRenderContext,
  ChunkUkey, CodeGenerationPublicPathAutoReplace, Compilation, ConcatenatedModuleIdent,
  ConditionalInitFragment, DependencyType, ExportInfoHashKey, ExportMode, ExportProvided,
  ExportsInfoArtifact, ExportsInfoGetter, ExportsType, FindTargetResult, GetUsedNameParam,
  ImportSpec, InitFragmentKey, ModuleGraph, ModuleGraphCacheArtifact, ModuleIdentifier, ModuleInfo,
  NAMESPACE_OBJECT_EXPORT, PathData, PrefetchExportsInfoMode, RuntimeGlobals,
  SideEffectsStateArtifact, SourceType, URLStaticMode, UsageState, UsedName, UsedNameItem,
  collect_ident, escape_name_atom_ref, find_new_name, find_target, get_cached_readable_identifier,
  get_js_chunk_filename_template, get_module_directives, get_module_hashbang, property_access,
  property_name, reserved_names::RESERVED_NAMES, rspack_sources::ReplaceSource,
  split_readable_identifier, to_normal_comment,
};
use rspack_error::{Diagnostic, Error, Result};
use rspack_plugin_javascript::{
  JsPlugin, RenderSource, dependency::ESMExportImportedSpecifierDependency,
};
use rspack_plugin_runtime::should_export_webpack_require_for_module_chunk_loading;
use rspack_util::{
  SpanExt,
  atom::Atom,
  fx_hash::{FxHashMap, FxHashSet, FxIndexMap, FxIndexSet},
};
use swc_core::common::{SyntaxContext, comments::SingleThreadedComments};
use swc_experimental_ecma_ast::{Ast, EsVersion, StringAllocator};
use swc_experimental_ecma_parser::{EsSyntax, Parser, StringSource, Syntax};
use swc_experimental_ecma_semantic::resolver::resolver;

use crate::{
  EsmLibraryPlugin,
  chunk_link::{ChunkLinkContext, ExternalInterop, RawImportSource, ReExportFrom, Ref, SymbolRef},
};

pub(crate) trait GetMut<K, V> {
  fn get_mut_unwrap(&mut self, key: &K) -> &mut V;
}

impl<K, V, S> GetMut<K, V> for collections::HashMap<K, V, S>
where
  K: Eq + std::hash::Hash,
  S: BuildHasher,
{
  fn get_mut_unwrap(&mut self, key: &K) -> &mut V {
    self.get_mut(key).expect("should have value in the map")
  }
}

impl<V> GetMut<ModuleIdentifier, V> for IdentifierIndexMap<V> {
  fn get_mut_unwrap(&mut self, key: &ModuleIdentifier) -> &mut V {
    self.get_mut(key).expect("should have value in the map")
  }
}

static START_EXPORTS: LazyLock<Atom> = LazyLock::new(|| "*".into());
#[derive(Default, Debug)]
pub(crate) struct ExportsContext {
  exports: FxHashMap<Atom, FxIndexSet<Atom>>,
  exported_symbols: FxHashSet<Atom>,
  re_exports: FxIndexMap<ReExportFrom, FxHashMap<Atom, FxHashSet<Atom>>>,
}

enum ExternalImportBinding {
  Default,
  Named(Atom),
  Namespace,
}

impl EsmLibraryPlugin {
  fn module_external_fragment_content(
    init_fragment: &Box<dyn rspack_core::InitFragment<ChunkRenderContext>>,
  ) -> Option<String> {
    if !matches!(init_fragment.key(), InitFragmentKey::ModuleExternal(_)) {
      return None;
    }

    if let Ok(fragment) = init_fragment
      .clone()
      .into_any()
      .downcast::<ConditionalInitFragment>()
    {
      Some(fragment.content().to_owned())
    } else {
      init_fragment
        .clone()
        .contents(&mut ChunkRenderContext {})
        .ok()
        .map(|contents| contents.start)
    }
  }

  fn collect_module_external_fragments_in_render_order<'a>(
    init_fragment_groups: impl IntoIterator<Item = &'a ChunkInitFragments>,
  ) -> Vec<String> {
    let mut ordered_fragments = Vec::new();

    for init_fragments in init_fragment_groups {
      for init_fragment in init_fragments {
        let Some(content) = Self::module_external_fragment_content(init_fragment) else {
          continue;
        };

        ordered_fragments.push((
          init_fragment.stage(),
          init_fragment.position(),
          ordered_fragments.len(),
          init_fragment.key().clone(),
          content,
        ));
      }
    }

    ordered_fragments.sort_by(|a, b| {
      let stage = a.0.cmp(&b.0);
      if !stage.is_eq() {
        return stage;
      }
      let position = a.1.cmp(&b.1);
      if !position.is_eq() {
        return position;
      }
      a.2.cmp(&b.2)
    });

    let mut rendered_keys = FxHashSet::default();
    let mut rendered_fragments = Vec::with_capacity(ordered_fragments.len());
    for (_, _, _, key, content) in ordered_fragments {
      if rendered_keys.insert(key) {
        rendered_fragments.push(content);
      }
    }

    rendered_fragments
  }

  fn parse_module_external_namespace_import(content: &str) -> Option<(RawImportSource, Atom)> {
    let content = content.trim_start();
    let content = content.strip_prefix("import * as ")?;
    let (local_name, source_clause) = content.split_once(" from ")?;
    if local_name.is_empty() {
      return None;
    }

    let source_clause = source_clause
      .lines()
      .next()
      .map(str::trim)
      .map(|line| line.trim_end_matches(';'))?;
    let (source_literal, attr) =
      if let Some((source_literal, attr)) = source_clause.split_once(" with ") {
        (source_literal, Some(format!(" with {attr}")))
      } else {
        (source_clause, None)
      };
    let source = serde_json::from_str::<String>(source_literal).ok()?;
    Some((RawImportSource::Source((source, attr)), local_name.into()))
  }

  fn collect_module_external_namespace_imports(
    init_fragments: &ChunkInitFragments,
  ) -> Vec<(RawImportSource, Atom)> {
    Self::collect_module_external_namespace_imports_in_render_order([init_fragments])
  }

  #[cfg(test)]
  fn reserve_module_external_namespace_import_locals(
    init_fragments: &ChunkInitFragments,
    used_names: &mut FxHashSet<Atom>,
    namespace_imports: Option<&mut FxHashMap<RawImportSource, Atom>>,
  ) {
    Self::reserve_module_external_namespace_import_locals_in_render_order(
      [init_fragments],
      used_names,
      namespace_imports,
    );
  }

  fn collect_module_external_namespace_imports_in_render_order<'a>(
    init_fragment_groups: impl IntoIterator<Item = &'a ChunkInitFragments>,
  ) -> Vec<(RawImportSource, Atom)> {
    Self::collect_module_external_fragments_in_render_order(init_fragment_groups)
      .into_iter()
      .filter_map(|content| Self::parse_module_external_namespace_import(&content))
      .collect()
  }

  fn reserve_module_external_namespace_import_locals_in_render_order<'a>(
    init_fragment_groups: impl IntoIterator<Item = &'a ChunkInitFragments>,
    used_names: &mut FxHashSet<Atom>,
    namespace_imports: Option<&mut FxHashMap<RawImportSource, Atom>>,
  ) {
    let mut namespace_imports = namespace_imports;
    for (source, local_name) in
      Self::collect_module_external_namespace_imports_in_render_order(init_fragment_groups)
    {
      if let Some(namespace_imports) = namespace_imports.as_mut() {
        if namespace_imports.contains_key(&source) {
          continue;
        }
        used_names.insert(local_name.clone());
        namespace_imports.insert(source, local_name);
      } else {
        used_names.insert(local_name);
      }
    }
  }

  fn strip_leading_comments(mut line: &str) -> &str {
    loop {
      line = line.trim_start();

      if line.is_empty() {
        return line;
      }

      if let Some(rest) = line.strip_prefix("//") {
        let _ = rest;
        return "";
      }

      if let Some(rest) = line.strip_prefix("/*")
        && let Some(comment_end) = rest.find("*/")
      {
        line = &rest[comment_end + 2..];
        continue;
      }

      return line;
    }
  }

  fn parse_identifier(input: &str) -> Option<(&str, &str)> {
    let input = input.trim_start();
    let mut chars = input.char_indices();
    let (_, first) = chars.next()?;
    if !(first == '_' || first == '$' || first.is_ascii_alphabetic()) {
      return None;
    }

    let mut end = first.len_utf8();
    for (idx, ch) in chars {
      if ch == '_' || ch == '$' || ch.is_ascii_alphanumeric() {
        end = idx + ch.len_utf8();
      } else {
        break;
      }
    }

    Some((&input[..end], &input[end..]))
  }

  fn parse_named_import_locals(clause: &str) -> Vec<Atom> {
    let Some(clause) = clause.strip_prefix('{') else {
      return vec![];
    };
    let Some((named_part, _)) = clause.split_once('}') else {
      return vec![];
    };

    named_part
      .split(',')
      .filter_map(|specifier| {
        let specifier = specifier.trim();
        if specifier.is_empty() {
          return None;
        }

        let local = specifier
          .rsplit_once(" as ")
          .map(|(_, local)| local)
          .unwrap_or(specifier)
          .trim();

        if local.is_empty() {
          None
        } else {
          Some(local.into())
        }
      })
      .collect()
  }

  fn parse_module_external_top_level_decls(content: &str) -> Vec<Atom> {
    let mut decls = Vec::new();

    for line in content.lines() {
      let line = Self::strip_leading_comments(line);
      if line.is_empty() {
        continue;
      }

      if let Some(import_clause) = line.strip_prefix("import ")
        && let Some((binding_clause, _)) = import_clause.rsplit_once(" from ")
      {
        let binding_clause = binding_clause.trim();
        if binding_clause.starts_with('"') || binding_clause.starts_with('\'') {
          continue;
        }

        if let Some(namespace_clause) = binding_clause.strip_prefix("* as ") {
          if let Some((local, _)) = Self::parse_identifier(namespace_clause) {
            decls.push(local.into());
          }
          continue;
        }

        if let Some((default_clause, rest)) = binding_clause.split_once(',') {
          if let Some((local, _)) = Self::parse_identifier(default_clause) {
            decls.push(local.into());
          }

          let rest = rest.trim();
          if let Some(namespace_clause) = rest.strip_prefix("* as ") {
            if let Some((local, _)) = Self::parse_identifier(namespace_clause) {
              decls.push(local.into());
            }
          } else {
            decls.extend(Self::parse_named_import_locals(rest));
          }
          continue;
        }

        if binding_clause.starts_with('{') {
          decls.extend(Self::parse_named_import_locals(binding_clause));
          continue;
        }

        if let Some((local, _)) = Self::parse_identifier(binding_clause) {
          decls.push(local.into());
        }

        continue;
      }

      for keyword in ["const ", "let ", "var "] {
        if let Some(rest) = line.strip_prefix(keyword) {
          if let Some((local, _)) = Self::parse_identifier(rest) {
            decls.push(local.into());
          }
          break;
        }
      }
    }

    decls
  }

  #[cfg(test)]
  fn reserve_module_external_top_level_decls(
    init_fragments: &ChunkInitFragments,
    used_names: &mut FxHashSet<Atom>,
  ) {
    Self::reserve_module_external_top_level_decls_in_render_order([init_fragments], used_names);
  }

  fn reserve_module_external_top_level_decls_in_render_order<'a>(
    init_fragment_groups: impl IntoIterator<Item = &'a ChunkInitFragments>,
    used_names: &mut FxHashSet<Atom>,
  ) {
    for content in Self::collect_module_external_fragments_in_render_order(init_fragment_groups) {
      used_names.extend(Self::parse_module_external_top_level_decls(&content));
    }
  }

  fn assign_external_candidate_name(
    readable_identifier: &str,
    candidate_used_names: &mut FxHashSet<Atom>,
    escaped_identifiers: &FxHashMap<String, Vec<Atom>>,
  ) -> Atom {
    let name = find_new_name(
      "",
      candidate_used_names,
      &escaped_identifiers[readable_identifier],
    );
    candidate_used_names.insert(name.clone());
    name
  }

  fn strict_export_chunk(&self, chunk: ChunkUkey) -> bool {
    self.strict_export_chunks.borrow().contains(&chunk)
  }

  fn add_chunk_export(
    chunk: ChunkUkey,
    local: Atom,
    exported: Atom,
    chunk_exports: &mut FxHashMap<ChunkUkey, ExportsContext>,
    strict_exports: bool,
  ) -> Option<Atom> {
    let ctx = chunk_exports.get_mut_unwrap(&chunk);
    if !strict_exports
      && let Some(already_exported_names) = ctx.exports.get(&local)
      && !already_exported_names.is_empty()
    {
      return Some(
        already_exported_names
          .iter()
          .next()
          .expect("should have export name")
          .clone(),
      );
    }

    // we've not exported this local symbol, check if we've already exported this symbol
    if ctx.exported_symbols.contains(&exported) {
      // the name is already exported and we know the exported_local is not the same
      if strict_exports {
        if let Some(already_exported_names) = ctx.exports.get(&local)
          && already_exported_names.contains(&exported)
        {
          return Some(exported);
        }

        return None;
      }

      let already_exported_names = ctx.exports.entry(local).or_default();

      // we find another name to export this symbol
      let mut idx = 0;
      let mut new_export = Atom::new(format!("{exported}_{idx}"));
      while ctx.exported_symbols.contains(&new_export) {
        idx += 1;
        new_export = format!("{exported}_{idx}").into();
      }

      ctx.exported_symbols.insert(new_export.clone());
      already_exported_names.insert(new_export.clone());
      already_exported_names.get(&new_export).cloned()
    } else {
      let already_exported_names = ctx.exports.entry(local).or_default();
      ctx.exported_symbols.insert(exported.clone());
      already_exported_names.insert(exported.clone());
      already_exported_names.get(&exported).cloned()
    }
  }

  // // orig_chunk
  // export { local_name as export_name } from 'ref_chunk'
  fn add_chunk_re_export(
    orig_chunk: ChunkUkey,
    ref_chunk: ChunkUkey,
    local_name: Atom,
    export_name: Atom,
    chunk_exports: &mut FxHashMap<ChunkUkey, ExportsContext>,
    strict_exports: bool,
  ) -> Option<&Atom> {
    let exports_context = chunk_exports.get_mut_unwrap(&orig_chunk);

    let export_name = if !exports_context.exported_symbols.contains(&export_name) {
      export_name
    } else {
      if strict_exports {
        return None;
      }
      let mut idx = 0;
      let mut new_export = Atom::new(format!("{export_name}_{idx}"));
      while exports_context.exported_symbols.contains(&new_export) {
        idx += 1;
        new_export = format!("{export_name}_{idx}").into();
      }
      new_export
    };

    exports_context.exported_symbols.insert(export_name.clone());

    let set = exports_context
      .re_exports
      .entry(ReExportFrom::Chunk(ref_chunk))
      .or_default()
      .entry(local_name)
      .or_default();

    set.insert(export_name.clone());
    Some(set.get(&export_name).expect("should have inserted"))
  }

  fn add_re_export_from_request(
    chunk: ChunkUkey,
    request: String,
    imported_name: Atom,
    export_name: Atom,
    chunk_exports: &mut FxHashMap<ChunkUkey, ExportsContext>,
  ) {
    let ctx = chunk_exports.get_mut_unwrap(&chunk);
    ctx.exported_symbols.insert(export_name.clone());

    ctx
      .re_exports
      .entry(ReExportFrom::Request(request))
      .or_default()
      .entry(imported_name)
      .or_default()
      .insert(export_name);
  }

  fn get_external_import_source_and_binding(
    info: &rspack_core::ConcatenatedModuleInfo,
    local_name: &Atom,
  ) -> Option<(RawImportSource, ExternalImportBinding)> {
    if let Some(import_map) = info.import_map.as_ref() {
      for ((source, attr), imported_atoms) in import_map {
        let raw_import_source = RawImportSource::Source((source.clone(), attr.clone()));

        if imported_atoms
          .namespace
          .as_ref()
          .is_some_and(|namespace| namespace == local_name)
          || info.namespace_object_name.as_ref() == Some(local_name)
        {
          return Some((raw_import_source, ExternalImportBinding::Namespace));
        }

        for imported_name in &imported_atoms.specifiers {
          let internal_name = info
            .get_internal_name(imported_name)
            .unwrap_or(imported_name);
          if internal_name != local_name {
            continue;
          }

          return Some((
            raw_import_source,
            if imported_name == "default" {
              ExternalImportBinding::Default
            } else {
              ExternalImportBinding::Named(imported_name.clone())
            },
          ));
        }
      }
    }

    if info.namespace_object_name.as_ref() == Some(local_name) {
      return Self::collect_module_external_namespace_imports(&info.chunk_init_fragments)
        .into_iter()
        .next()
        .map(|(source, _)| (source, ExternalImportBinding::Namespace));
    }

    None
  }

  pub(crate) async fn link(
    &self,
    compilation: &Compilation,
    diagnostics: &mut Vec<Diagnostic>,
  ) -> Result<()> {
    let module_graph = compilation.get_module_graph();

    // codegen uses self.concatenated_modules_map_for_codegen which has hold another Arc, so
    // it's safe to access concate_modules_map lock
    let mut concate_modules_map = self.concatenated_modules_map.write().await;
    let mut external_module_init_fragments = IdentifierMap::default();

    // analyze every modules and collect identifiers to concate_modules_map
    self
      .analyze_module(
        compilation,
        &mut concate_modules_map,
        &mut external_module_init_fragments,
      )
      .await?;

    // initialize data for link chunks
    let mut link: FxHashMap<ChunkUkey, ChunkLinkContext> = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .keys()
      .map(|ukey| {
        let modules = compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .get_chunk_modules_identifier(ukey);

        let mut decl_modules = IdentifierIndexSet::default();
        let mut hoisted_modules = IdentifierIndexSet::default();

        for m in modules.iter() {
          if compilation
            .code_generation_results
            .get_one(m)
            .get(&SourceType::JavaScript)
            .is_none()
          {
            continue;
          }

          let info = concate_modules_map
            .get(m)
            .unwrap_or_else(|| panic!("should have set module info for {m}"));

          if matches!(info, ModuleInfo::Concatenated(_)) {
            hoisted_modules.insert(*m);
          } else {
            decl_modules.insert(*m);
          }
        }

        // sort by module identifier to get better gzip size, as similar identifiers
        // means more probably they are in the same directory, and the code is more
        // likely to be similar.
        decl_modules.sort_unstable();

        // sort scope-hoisted modules based on the post order index
        hoisted_modules.sort_by(|m1, m2| {
          let m1_index = module_graph.get_post_order_index(m1);
          let m2_index = module_graph.get_post_order_index(m2);
          m1_index.cmp(&m2_index)
        });

        let chunk_link = ChunkLinkContext::new(*ukey, hoisted_modules, decl_modules);

        (*ukey, chunk_link)
      })
      .collect();

    let runtime_chunks_exporting_require_via_runtime_module = link
      .keys()
      .copied()
      .filter(|chunk_ukey| {
        should_export_webpack_require_for_module_chunk_loading(chunk_ukey, compilation)
      })
      .collect::<FxHashSet<_>>();

    for runtime_chunk in &runtime_chunks_exporting_require_via_runtime_module {
      link
        .get_mut_unwrap(runtime_chunk)
        .exports_require_via_runtime_module = true;
    }

    let (escaped_name_entries, escaped_identifier_entries) = concate_modules_map
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
          &compilation.module_static_cache,
          &compilation.options.context,
        );
        let splitted_readable_identifier = split_readable_identifier(&readable_identifier);
        escaped_identifiers.push((readable_identifier, splitted_readable_identifier));

        match info {
          ModuleInfo::Concatenated(info) => {
            for (id, _) in info.binding_to_ref.iter() {
              escaped_names
                .entry(id.0.clone())
                .or_insert_with(|| escape_name_atom_ref(&id.0));
            }

            if let Some(import_map) = &info.import_map {
              for ((source, _), imported_atoms) in import_map.iter() {
                escaped_identifiers
                  .push((source.clone(), split_readable_identifier(source.as_str())));
                for atom in &imported_atoms.specifiers {
                  escaped_names
                    .entry(atom.clone())
                    .or_insert_with(|| escape_name_atom_ref(atom));
                }
                if let Some(ns_import) = &imported_atoms.namespace {
                  escaped_names
                    .entry(ns_import.clone())
                    .or_insert_with(|| escape_name_atom_ref(ns_import));
                }
              }
            }
          }
          ModuleInfo::External(_) => (),
        }
        (
          escaped_names.into_iter().collect::<Vec<_>>(),
          escaped_identifiers,
        )
      })
      .reduce(
        || (Vec::new(), Vec::new()),
        |mut a, mut b| {
          a.0.append(&mut b.0);
          a.1.append(&mut b.1);
          a
        },
      );
    let mut escaped_names =
      FxHashMap::with_capacity_and_hasher(escaped_name_entries.len(), Default::default());
    for (name, escaped_name) in escaped_name_entries {
      escaped_names.insert(name, escaped_name);
    }
    let mut escaped_identifiers =
      FxHashMap::with_capacity_and_hasher(escaped_identifier_entries.len(), Default::default());
    for (identifier, parts) in escaped_identifier_entries {
      escaped_identifiers.insert(identifier, parts);
    }

    for chunk_link in link.values_mut() {
      self.deconflict_symbols(
        compilation,
        &mut concate_modules_map,
        &external_module_init_fragments,
        chunk_link,
        &escaped_names,
        &escaped_identifiers,
      );
    }

    // link imported specifier with exported symbol
    let mut needed_namespace_objects_by_ukey = FxHashMap::default();
    diagnostics.extend(self.link_imports_and_exports(
      compilation,
      &mut link,
      &runtime_chunks_exporting_require_via_runtime_module,
      &mut concate_modules_map,
      &mut needed_namespace_objects_by_ukey,
      &escaped_identifiers,
    ));

    let mut namespace_object_sources: IdentifierMap<String> = IdentifierMap::default();
    let mut namespace_re_export_star_cache = IdentifierMap::default();
    for (ukey, mut needed_namespace_objects) in needed_namespace_objects_by_ukey {
      let mut visited = FxHashSet::default();

      let chunk_link = link.get_mut_unwrap(&ukey);

      // webpack require iterate the needed_namespace_objects and mutate `needed_namespace_objects`
      // at the same time, https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/ConcatenatedModule.js#L1514
      // Which is impossible in rust, using a fixed point algorithm  to reach the same goal.
      loop {
        let mut changed = false;

        // using the previous round snapshot `needed_namespace_objects` to iterate, and modify the
        // original `needed_namespace_objects` during the iteration,
        // if there is no new id inserted into `needed_namespace_objects`, break the outer loop
        for module_info_id in needed_namespace_objects.clone().iter() {
          if visited.contains(module_info_id) {
            continue;
          }
          visited.insert(*module_info_id);
          changed = true;

          let module_info = concate_modules_map[module_info_id].as_concatenated();
          let mut runtime_template = compilation.runtime_template.create_module_code_template();

          let module_graph = compilation.get_module_graph();
          let box_module = module_graph
            .module_by_identifier(module_info_id)
            .expect("should have box module");
          let module_readable_identifier =
            box_module.readable_identifier(&compilation.options.context);
          let strict_esm_module = box_module.build_meta().strict_esm_module;

          // scope hoisted module can only exist in only 1 chunk
          // so it's safe to use module_info.namespace_object_name, which is
          // already de-conflicted
          let namespace_name = module_info.namespace_object_name.clone();

          if module_info.namespace_export_symbol.is_some() {
            continue;
          }

          let mut ns_obj = Vec::new();
          for export_info in compilation
            .exports_info_artifact
            .get_exports_info_data(module_info_id)
            .exports()
            .values()
          {
            if matches!(export_info.provided(), Some(ExportProvided::NotProvided)) {
              continue;
            }

            if matches!(export_info.get_used(None), UsageState::Unused) {
              // skip useless
              continue;
            }

            if let Some(UsedNameItem::Str(used_name)) = export_info.get_used_name(None, None) {
              let Some(mut binding) = Self::get_binding(
                None,
                compilation.get_module_graph(),
                &compilation.module_graph_cache_artifact,
                &compilation.exports_info_artifact,
                module_info_id,
                vec![export_info.name().cloned().unwrap_or("".into())],
                &mut concate_modules_map,
                &mut needed_namespace_objects,
                false,
                false,
                strict_esm_module,
                Some(true),
                &mut Default::default(),
                &mut chunk_link.required,
                &mut chunk_link.used_names,
              ) else {
                continue;
              };

              if let Ref::Symbol(symbol_binding) = &mut binding {
                let target_info = concate_modules_map.get(&symbol_binding.module);
                if matches!(target_info, Some(ModuleInfo::External(_))) {
                  chunk_link.imports.entry(symbol_binding.module).or_default();
                } else if symbol_binding.ids.is_empty()
                  && matches!(target_info, Some(ModuleInfo::Concatenated(_)))
                {
                  // For module-type externals stored as Concatenated, the raw_export_map
                  // returns bare symbol names (e.g., "readFile") that aren't local variables.
                  // When ids is empty, the symbol is directly referenced (not via property
                  // access like ns.readFile), so we need to add an import from the external
                  // source to make the binding available.
                  let module_graph = compilation.get_module_graph();
                  if let Some(ext) = module_graph
                    .module_by_identifier(&symbol_binding.module)
                    .and_then(|m| m.as_external_module())
                    && ext.get_external_type().as_str().starts_with("module")
                  {
                    let Some((raw_import_source, import_binding)) = concate_modules_map
                      .get(&symbol_binding.module)
                      .and_then(|target_info| match target_info {
                        ModuleInfo::Concatenated(target_info) => {
                          Self::get_external_import_source_and_binding(
                            target_info,
                            &symbol_binding.symbol,
                          )
                        }
                        ModuleInfo::External(_) => None,
                      })
                    else {
                      continue;
                    };

                    if matches!(import_binding, ExternalImportBinding::Namespace)
                      && let Some(existing_local) = chunk_link
                        .module_external_namespace_imports
                        .get(&raw_import_source)
                    {
                      symbol_binding.symbol = existing_local.clone();
                      continue;
                    }

                    let import_spec = chunk_link
                      .raw_import_stmts
                      .entry(raw_import_source)
                      .or_default();

                    let existing_local = match &import_binding {
                      ExternalImportBinding::Default => import_spec.default_import.as_ref(),
                      ExternalImportBinding::Named(imported_name) => {
                        import_spec.atoms.get(imported_name)
                      }
                      ExternalImportBinding::Namespace => import_spec.ns_import.as_ref(),
                    };

                    if let Some(existing_local) = existing_local {
                      symbol_binding.symbol = existing_local.clone();
                    } else {
                      let local_name = if chunk_link.used_names.contains(&symbol_binding.symbol) {
                        let new_name = find_new_name(
                          symbol_binding.symbol.as_str(),
                          &chunk_link.used_names,
                          &[],
                        );
                        chunk_link.used_names.insert(new_name.clone());
                        new_name
                      } else {
                        let local_name = symbol_binding.symbol.clone();
                        chunk_link.used_names.insert(local_name.clone());
                        local_name
                      };

                      match import_binding {
                        ExternalImportBinding::Default => {
                          import_spec.default_import = Some(local_name.clone());
                        }
                        ExternalImportBinding::Named(imported_name) => {
                          import_spec.atoms.insert(imported_name, local_name.clone());
                        }
                        ExternalImportBinding::Namespace => {
                          import_spec.ns_import = Some(local_name.clone());
                        }
                      }
                      symbol_binding.symbol = local_name;
                    }
                  }
                }
              }

              ns_obj.push(format!(
                "\n  {}: {}",
                property_name(&used_name).expect("should have property_name"),
                runtime_template.returning_function(&binding.render(), "")
              ));
            }
          }
          let name = namespace_name.expect("should have name_space_name");
          let star_re_export_binding = Self::resolve_single_star_re_export_target(
            *module_info_id,
            module_graph,
            &compilation.module_graph_cache_artifact,
            &compilation
              .build_module_graph_artifact
              .side_effects_state_artifact,
            &compilation.exports_info_artifact,
            &mut namespace_re_export_star_cache,
          )
          .and_then(|target_module| {
            let target_strict_esm_module = module_graph
              .module_by_identifier(&target_module)
              .expect("should have target module")
              .build_meta()
              .strict_esm_module;

            Self::get_binding(
              None,
              module_graph,
              &compilation.module_graph_cache_artifact,
              &compilation.exports_info_artifact,
              &target_module,
              vec![],
              &mut concate_modules_map,
              &mut needed_namespace_objects,
              false,
              false,
              target_strict_esm_module,
              None,
              &mut Default::default(),
              &mut chunk_link.required,
              &mut chunk_link.used_names,
            )
          });
          let star_re_export_getters = if let Some(binding) = star_re_export_binding {
            let star_exports_base = format!("{name}_starExports");
            let star_exports_name =
              find_new_name(star_exports_base.as_str(), &chunk_link.used_names, &[]);
            chunk_link.used_names.insert(star_exports_name.clone());

            format!(
              r#"var {} = {};
Object.keys({}).forEach(function(key) {{
  if (key !== "default" && key !== "__esModule") {{
    {}({}, {{ [key]: function() {{ return {}[key]; }} }});
  }}
}});
"#,
              star_exports_name,
              binding.render(),
              star_exports_name,
              runtime_template.render_runtime_globals(&RuntimeGlobals::DEFINE_PROPERTY_GETTERS),
              name,
              star_exports_name
            )
          } else {
            String::new()
          };
          let define_getters = if !ns_obj.is_empty() {
            format!(
              "{}({}, {{ {} }});\n",
              runtime_template.render_runtime_globals(&RuntimeGlobals::DEFINE_PROPERTY_GETTERS),
              name,
              ns_obj.join(",")
            )
          } else {
            String::new()
          };

          let namespace_object_source =
            if !define_getters.is_empty() || !star_re_export_getters.is_empty() {
              format!(
                r#"// NAMESPACE OBJECT: {}
var {} = {{}};
{}({});
{}{}
"#,
                module_readable_identifier,
                name,
                runtime_template.render_runtime_globals(&RuntimeGlobals::MAKE_NAMESPACE_OBJECT),
                name,
                define_getters,
                star_re_export_getters
              )
            } else {
              format!(
                r#"// NAMESPACE OBJECT: {}
var {} = {{}};
{}({});
{}
"#,
                module_readable_identifier,
                name,
                runtime_template.render_runtime_globals(&RuntimeGlobals::MAKE_NAMESPACE_OBJECT),
                name,
                define_getters
              )
            };

          let module_info = concate_modules_map[module_info_id].as_concatenated_mut();

          namespace_object_sources.insert(*module_info_id, namespace_object_source);

          module_info
            .runtime_requirements
            .insert(*runtime_template.runtime_requirements());
        }
        if !changed {
          break;
        }
      }
    }

    for (module, source) in namespace_object_sources {
      // Skip orphan modules — they are not in any chunk
      if compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_module_chunks(module)
        .is_empty()
      {
        continue;
      }
      let chunk = Self::get_module_chunk(module, compilation)?;
      let chunk_link = link.get_mut_unwrap(&chunk);
      chunk_link.namespace_object_sources.insert(module, source);
    }

    let mut links = self.links.borrow_mut();
    *links = link;
    Ok(())
  }

  pub(crate) fn get_module_chunk(
    m: ModuleIdentifier,
    compilation: &Compilation,
  ) -> rspack_error::Result<ChunkUkey> {
    let chunks = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_module_chunks(m);
    Self::validate_single_chunk(m, chunks)
  }

  fn validate_single_chunk(
    m: ModuleIdentifier,
    chunks: &FxHashSet<ChunkUkey>,
  ) -> rspack_error::Result<ChunkUkey> {
    match chunks.len() {
      0 => Err(rspack_error::error!("module {m} is not in any chunk")),
      1 => Ok(*chunks.iter().next().expect("chunks.len() == 1")),
      _ => Err(rspack_error::error!("module {m} is in multiple chunks")),
    }
  }

  fn deconflict_symbols(
    &self,
    compilation: &Compilation,
    concate_modules_map: &mut IdentifierIndexMap<ModuleInfo>,
    external_module_init_fragments: &IdentifierMap<ChunkInitFragments>,
    chunk_link: &mut ChunkLinkContext,
    escaped_names: &FxHashMap<Atom, Atom>,
    escaped_identifiers: &FxHashMap<String, Vec<Atom>>,
  ) {
    let context = &compilation.options.context;

    let module_graph = compilation.get_module_graph();

    let mut all_used_names: FxHashSet<Atom> = RESERVED_NAMES
      .iter()
      .map(|s| Atom::new(*s))
      .chain(chunk_link.hoisted_modules.iter().flat_map(|m| {
        let info = &concate_modules_map[m];
        info
          .as_concatenated()
          .global_scope_ident
          .iter()
          .map(|ident| ident.id.sym.clone())
      }))
      .collect();

    // merge all all_used_names from hoisted modules
    for id in &chunk_link.hoisted_modules {
      let concate_info = concate_modules_map[id].as_concatenated();
      all_used_names.extend(concate_info.all_used_names.iter().cloned());
    }

    // Pre-reserve namespace object names from dyn_import_ns_map so other
    // symbols don't claim these names during deconflict
    {
      let ns_map = self.dyn_import_ns_map.borrow();
      for id in chunk_link
        .hoisted_modules
        .iter()
        .chain(chunk_link.decl_modules.iter())
      {
        if let Some(ns_name) = ns_map.get(id) {
          all_used_names.insert(ns_name.clone());
        }
      }
    }

    let mut module_external_init_fragment_groups = vec![&chunk_link.init_fragments];
    for id in &chunk_link.decl_modules {
      match &concate_modules_map[id] {
        ModuleInfo::Concatenated(info) => {
          module_external_init_fragment_groups.push(&info.chunk_init_fragments);
        }
        ModuleInfo::External(info) => {
          if let Some(init_fragments) = external_module_init_fragments.get(&info.module) {
            module_external_init_fragment_groups.push(init_fragments);
          }
        }
      }
    }
    for id in &chunk_link.hoisted_modules {
      match &concate_modules_map[id] {
        ModuleInfo::Concatenated(info) => {
          module_external_init_fragment_groups.push(&info.chunk_init_fragments);
        }
        ModuleInfo::External(info) => {
          if let Some(init_fragments) = external_module_init_fragments.get(&info.module) {
            module_external_init_fragment_groups.push(init_fragments);
          }
        }
      }
    }
    Self::reserve_module_external_namespace_import_locals_in_render_order(
      module_external_init_fragment_groups.iter().copied(),
      &mut all_used_names,
      Some(&mut chunk_link.module_external_namespace_imports),
    );
    Self::reserve_module_external_top_level_decls_in_render_order(
      module_external_init_fragment_groups.iter().copied(),
      &mut all_used_names,
    );

    // deconflict top level symbols
    for id in chunk_link
      .hoisted_modules
      .iter()
      .chain(chunk_link.decl_modules.iter())
    {
      let module = module_graph
        .module_by_identifier(id)
        .expect("should have module");
      let exports_type = module.build_meta().exports_type;
      let default_object = module.build_meta().default_object;

      let info = &mut concate_modules_map[id];
      let readable_identifier =
        get_cached_readable_identifier(id, module_graph, &compilation.module_static_cache, context);

      if let ModuleInfo::Concatenated(concate_info) = info {
        let mut internal_names = FxHashMap::default();

        // registered import map
        if let Some(import_map) = &concate_info.import_map {
          for ((source, attr), imported_atoms) in import_map.iter() {
            let raw_import_source = RawImportSource::Source((source.clone(), attr.clone()));
            let existing_namespace_import = chunk_link
              .module_external_namespace_imports
              .get(&raw_import_source)
              .cloned();
            let mut total_imported_atoms = None;

            if imported_atoms.namespace.is_none()
              && imported_atoms.specifiers.is_empty()
              && existing_namespace_import.is_none()
            {
              total_imported_atoms = Some(
                chunk_link
                  .raw_import_stmts
                  .entry(raw_import_source.clone())
                  .or_default(),
              );
            }

            if let Some(ns_import) = &imported_atoms.namespace {
              if let Some(existing_local) = existing_namespace_import.as_ref() {
                if existing_local != ns_import {
                  internal_names.insert(ns_import.clone(), existing_local.clone());
                }
              } else {
                total_imported_atoms = Some(
                  chunk_link
                    .raw_import_stmts
                    .entry(raw_import_source.clone())
                    .or_default(),
                );
                total_imported_atoms
                  .as_mut()
                  .expect("should have import spec")
                  .ns_import = Some(ns_import.clone());
              }
            }

            for atom in &imported_atoms.specifiers {
              if total_imported_atoms.is_none() {
                total_imported_atoms = Some(
                  chunk_link
                    .raw_import_stmts
                    .entry(raw_import_source.clone())
                    .or_default(),
                );
              }
              let total_imported_atoms = total_imported_atoms
                .as_mut()
                .expect("should have import spec");
              // already import this symbol
              if let Some(internal_atom) = total_imported_atoms.atoms.get(atom).or_else(|| {
                if atom == "default"
                  && let Some(default_symbol) = &total_imported_atoms.default_import
                {
                  Some(default_symbol)
                } else {
                  None
                }
              }) {
                internal_names.insert(atom.clone(), internal_atom.clone());
                // if the imported symbol is exported, we rename the export as well
                if let Some(raw_export_map) = concate_info.raw_export_map.as_mut()
                  && raw_export_map.contains_key(atom)
                {
                  raw_export_map.insert(atom.clone(), internal_atom.to_string());
                }
                continue;
              }

              let new_name = if all_used_names.contains(atom) {
                let new_name = if atom == "default" {
                  find_new_name("", &all_used_names, &escaped_identifiers[source])
                } else {
                  find_new_name(
                    escaped_names
                      .get(atom)
                      .expect("should have escaped name")
                      .as_ref(),
                    &all_used_names,
                    &escaped_identifiers[&readable_identifier],
                  )
                };
                all_used_names.insert(new_name.clone());
                // if the imported symbol is exported, we rename the export as well
                if let Some(raw_export_map) = concate_info.raw_export_map.as_mut()
                  && raw_export_map.contains_key(atom)
                {
                  raw_export_map.insert(atom.clone(), new_name.to_string());
                }
                new_name
              } else {
                all_used_names.insert(atom.clone());
                atom.clone()
              };

              internal_names.insert(atom.clone(), new_name.clone());

              if atom == "default" {
                total_imported_atoms.default_import = Some(new_name.clone());
              } else {
                total_imported_atoms
                  .atoms
                  .insert(atom.clone(), new_name.clone());
              }
            }
          }
        }

        for (atom, ctxt) in concate_info.binding_to_ref.keys() {
          // only need to handle top level scope
          if ctxt != &concate_info.module_ctxt {
            continue;
          }

          if all_used_names.contains(atom) {
            let new_name = find_new_name(
              escaped_names
                .get(atom)
                .expect("should have escaped name")
                .as_ref(),
              &all_used_names,
              &escaped_identifiers[&readable_identifier],
            );

            all_used_names.insert(new_name.clone());
            internal_names.insert(atom.clone(), new_name);
          } else {
            all_used_names.insert(atom.clone());
            internal_names.insert(atom.clone(), atom.clone());
          }
        }

        concate_info.internal_names = internal_names;

        // Handle the name passed through by namespace_export_symbol
        if let Some(ref namespace_export_symbol) = concate_info.namespace_export_symbol
          && namespace_export_symbol.starts_with(NAMESPACE_OBJECT_EXPORT)
          && namespace_export_symbol.len() > NAMESPACE_OBJECT_EXPORT.len()
        {
          let name: Atom =
            Atom::from(namespace_export_symbol[NAMESPACE_OBJECT_EXPORT.len()..].to_string());
          all_used_names.insert(name.clone());
          concate_info
            .internal_names
            .insert(namespace_export_symbol.clone(), name.clone());
        }

        // Handle namespaceObjectName for concatenated type
        // If this module has a pre-assigned name from dyn_import_ns_map, use it directly
        let namespace_object_name = {
          let pre_assigned = {
            let ns_map = self.dyn_import_ns_map.borrow();
            ns_map.get(id).cloned()
          };
          if let Some(pre_assigned) = pre_assigned {
            pre_assigned
          } else if let Some(ref namespace_export_symbol) = concate_info.namespace_export_symbol {
            concate_info
              .get_internal_name(namespace_export_symbol)
              .cloned()
              .unwrap_or_else(|| {
                find_new_name(
                  "namespaceObject",
                  &all_used_names,
                  &escaped_identifiers[&readable_identifier],
                )
              })
          } else {
            find_new_name(
              "namespaceObject",
              &all_used_names,
              &escaped_identifiers[&readable_identifier],
            )
          }
        };
        all_used_names.insert(namespace_object_name.clone());
        concate_info.namespace_object_name = Some(namespace_object_name.clone());

        // Handle additional logic based on module build meta
        if exports_type != BuildMetaExportsType::Namespace {
          let external_name_interop: Atom = find_new_name(
            "namespaceObject",
            &all_used_names,
            &escaped_identifiers[&readable_identifier],
          );
          all_used_names.insert(external_name_interop.clone());
          info.set_interop_namespace_object_name(Some(external_name_interop.clone()));
        }

        if exports_type == BuildMetaExportsType::Default
          && !matches!(default_object, BuildMetaDefaultObject::Redirect)
        {
          let external_name_interop: Atom = find_new_name(
            "namespaceObject2",
            &all_used_names,
            &escaped_identifiers[&readable_identifier],
          );
          all_used_names.insert(external_name_interop.clone());
          info.set_interop_namespace_object2_name(Some(external_name_interop.clone()));
        }

        if matches!(
          exports_type,
          BuildMetaExportsType::Dynamic | BuildMetaExportsType::Unset
        ) {
          let external_name_interop: Atom = find_new_name(
            "default",
            &all_used_names,
            &escaped_identifiers[&readable_identifier],
          );
          all_used_names.insert(external_name_interop.clone());
          info.set_interop_default_access_name(Some(external_name_interop.clone()));
        }
      }
    }

    // Build a targeted set for external module name deconfliction:
    // Start from chunk_link.used_names (cross-chunk accumulated names) and add
    // names that will actually be emitted at the chunk top level. We intentionally
    // avoid using all_used_names directly because it also contains transient
    // binding_to_ref keys (e.g. `cjs`, `foo`) that are rewritten during rendering
    // and should not block external module names.
    let mut emitted_external_used_names = chunk_link.used_names.clone();
    for module in chunk_link
      .hoisted_modules
      .iter()
      .chain(chunk_link.decl_modules.iter())
    {
      match &concate_modules_map[module] {
        ModuleInfo::Concatenated(info) => {
          if let Some(name) = &info.namespace_object_name {
            emitted_external_used_names.insert(name.clone());
          }
          if info.interop_namespace_object_used
            && let Some(name) = &info.interop_namespace_object_name
          {
            emitted_external_used_names.insert(name.clone());
          }
          if info.interop_namespace_object2_used
            && let Some(name) = &info.interop_namespace_object2_name
          {
            emitted_external_used_names.insert(name.clone());
          }
          if info.interop_default_access_used
            && let Some(name) = &info.interop_default_access_name
          {
            emitted_external_used_names.insert(name.clone());
          }
        }
        ModuleInfo::External(info) => {
          if info.interop_namespace_object_used
            && let Some(name) = &info.interop_namespace_object_name
          {
            emitted_external_used_names.insert(name.clone());
          }
          if info.interop_namespace_object2_used
            && let Some(name) = &info.interop_namespace_object2_name
          {
            emitted_external_used_names.insert(name.clone());
          }
          if info.interop_default_access_used
            && let Some(name) = &info.interop_default_access_name
          {
            emitted_external_used_names.insert(name.clone());
          }
          if info.deferred
            && let Some(name) = &info.deferred_name
          {
            emitted_external_used_names.insert(name.clone());
          }
          if info.deferred_namespace_object_used
            && let Some(name) = &info.deferred_namespace_object_name
          {
            emitted_external_used_names.insert(name.clone());
          }
        }
      }
    }
    for import_spec in chunk_link.raw_import_stmts.values() {
      if let Some(ns) = &import_spec.ns_import {
        emitted_external_used_names.insert(ns.clone());
      }
      for atom in import_spec.atoms.values() {
        emitted_external_used_names.insert(atom.clone());
      }
      if let Some(default_import) = &import_spec.default_import {
        emitted_external_used_names.insert(default_import.clone());
      }
    }

    let mut external_candidate_used_names = emitted_external_used_names.clone();
    for external_module in chunk_link.decl_modules.iter() {
      let ModuleInfo::External(info) = &mut concate_modules_map[external_module] else {
        unreachable!("should be un-scope-hoisted module");
      };

      if info.name.is_none() {
        let readable_identifier = get_cached_readable_identifier(
          external_module,
          module_graph,
          &compilation.module_static_cache,
          context,
        );

        let name = Self::assign_external_candidate_name(
          &readable_identifier,
          &mut external_candidate_used_names,
          escaped_identifiers,
        );
        info.name = Some(name);
      }
    }

    all_used_names.extend(emitted_external_used_names);
    chunk_link.used_names = all_used_names;
  }

  async fn analyze_module(
    &self,
    compilation: &Compilation,
    orig_concate_modules_map: &mut IdentifierIndexMap<ModuleInfo>,
    external_module_init_fragments: &mut IdentifierMap<ChunkInitFragments>,
  ) -> Result<()> {
    let runtime_template = compilation.runtime_template.create_runtime_code_template();
    let mut outputs = FxHashMap::<ChunkUkey, String>::default();
    let module_keys: Vec<ModuleIdentifier> = orig_concate_modules_map.keys().copied().collect();
    for m in &module_keys {
      if compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_module_chunks(*m)
        .is_empty()
      {
        // orphan module
        continue;
      }

      let chunk_ukey = Self::get_module_chunk(*m, compilation)?;
      let chunk = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .expect_get(&chunk_ukey);
      let filename_template = get_js_chunk_filename_template(
        chunk,
        &compilation.options.output,
        &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
      );

      let output_path = compilation
        .get_path_with_info(
          &filename_template,
          PathData::default()
            .chunk_hash_optional(chunk.rendered_hash(
              &compilation.chunk_hashes_artifact,
              compilation.options.output.hash_digest_length,
            ))
            .chunk_id_optional(chunk.id().map(|id| id.as_str()))
            .chunk_name_optional(chunk.name_for_filename_template())
            .content_hash_optional(chunk.rendered_content_hash_by_source_type(
              &compilation.chunk_hashes_artifact,
              &SourceType::JavaScript,
              compilation.options.output.hash_digest_length,
            ))
            .runtime(chunk.runtime().as_str()),
          &mut Default::default(),
        )
        .await
        .expect("should have output path");
      outputs.insert(chunk_ukey, output_path);
    }

    let concate_modules_map = std::mem::take(orig_concate_modules_map);
    let map = rspack_parallel::scope::<_, _>(|token| {
      for (m, info) in concate_modules_map {
        // SAFETY: caller will poll the futures
        let s = unsafe { token.used((compilation, m, info, &runtime_template)) };
        s.spawn(
          async move |(compilation, id, info, runtime_template)| -> Result<(
            ModuleInfo,
            Option<(ModuleIdentifier, ChunkInitFragments)>,
          )> {
            if compilation
              .build_chunk_graph_artifact
              .chunk_graph
              .get_module_chunks(m)
              .is_empty()
            {
              // orphan module
              return Ok((info, None));
            }

            let chunk_ukey = Self::get_module_chunk(m, compilation)?;

            let module_graph = compilation.get_module_graph();

            match info {
              rspack_core::ModuleInfo::External(mut external_module_info) => {
                let codegen_res = compilation.code_generation_results.get(&id, None);
                let has_javascript_source = compilation
                  .code_generation_results
                  .get(&id, None)
                  .get(&SourceType::JavaScript)
                  .is_some();
                let used_in_chunk = !compilation
                  .build_chunk_graph_artifact
                  .chunk_graph
                  .get_module_chunks(id)
                  .is_empty();
                if has_javascript_source && used_in_chunk {
                  // we use __webpack_require__.add({...}) to register modules
                  external_module_info
                    .runtime_requirements
                    .insert(RuntimeGlobals::REQUIRE | RuntimeGlobals::MODULE_FACTORIES);
                }
                let mut chunk_init_fragments = codegen_res
                  .data
                  .get::<ChunkInitFragments>()
                  .cloned()
                  .unwrap_or_default();
                chunk_init_fragments.extend(codegen_res.chunk_init_fragments.clone());
                Ok((
                  ModuleInfo::External(external_module_info),
                  if chunk_init_fragments.is_empty() {
                    None
                  } else {
                    Some((id, chunk_init_fragments))
                  },
                ))
              }
              rspack_core::ModuleInfo::Concatenated(mut concate_info) => {
                let hooks = JsPlugin::get_compilation_hooks(compilation.id());
                let hooks = hooks.read().await;

                let codegen_res = compilation.code_generation_results.get(&id, None);
                let Some(js_source) = codegen_res.get(&SourceType::JavaScript) else {
                  return Ok((ModuleInfo::Concatenated(concate_info), None));
                };

                let mut render_source = RenderSource {
                  source: js_source.clone(),
                };

                let mut chunk_init_fragments = vec![];
                hooks
                  .render_module_content
                  .call(
                    compilation,
                    &chunk_ukey,
                    module_graph
                      .module_by_identifier(&m)
                      .expect("should have module")
                      .as_ref(),
                    &mut render_source,
                    &mut chunk_init_fragments,
                    runtime_template,
                  )
                  .await?;
                *concate_info = codegen_res
                  .concatenation_scope
                  .as_ref()
                  .expect("should have concatenation scope")
                  .current_module
                  .clone();

                let m = module_graph
                  .module_by_identifier(&id)
                  .expect("should have module");
                let jsx = m
                  .as_ref()
                  .as_normal_module()
                  .and_then(|normal_module| normal_module.get_parser_options())
                  .and_then(|options| {
                    options
                      .get_javascript()
                      .and_then(|js_options| js_options.jsx)
                  })
                  .unwrap_or(false);
                let source_str = render_source
                  .source
                  .source()
                  .into_string_lossy()
                  .into_owned();
                let comments = SingleThreadedComments::default();
                let mut ast = Ast::new(source_str.len(), StringAllocator::default());
                let lexer = swc_experimental_ecma_parser::Lexer::new(
                  Syntax::Es(EsSyntax {
                    jsx,
                    ..Default::default()
                  }),
                  EsVersion::EsNext,
                  StringSource::new(&source_str),
                  Some(&comments),
                  ast.string_allocator(),
                );
                let mut parser = Parser::new_from(&mut ast, lexer);
                let module = match parser.parse_module() {
                  Ok(module) => module,
                  Err(err) => {
                    return Err(Error::from_string(
                      Some(source_str.clone()),
                      err.span().real_lo() as usize,
                      err.span().real_hi() as usize,
                      "JavaScript parse error:\n".to_string(),
                      err.kind().msg().to_string(),
                    ));
                  }
                };
                let ast = &ast;
                let semantic = resolver(module, ast);
                let ids = collect_ident(ast, module);

                concate_info.module_ctxt = semantic.top_level_scope_id().to_ctxt();
                concate_info.global_ctxt = semantic.unresolved_scope_id().to_ctxt();

                let top_level_scope_id = semantic.top_level_scope_id();
                let mut all_used_names = FxHashSet::default();
                all_used_names.reserve(ids.len());
                concate_info.idents.reserve(ids.len());
                concate_info.global_scope_ident.reserve(ids.len());
                let mut binding_to_ref: FxIndexMap<
                  (Atom, SyntaxContext),
                  Vec<ConcatenatedModuleIdent>,
                > = Default::default();
                binding_to_ref.reserve(ids.len());

                for ident in ids {
                  let scope = semantic.node_scope(ident.id);
                  let is_global = scope.to_ctxt() == concate_info.global_ctxt;
                  let legacy = if is_global {
                    let leg = ident.to_legacy(ast, &semantic);
                    concate_info.global_scope_ident.push(leg.clone());
                    all_used_names.insert(leg.id.sym.clone());
                    Some(leg)
                  } else {
                    None
                  };
                  if ident.is_class_expr_with_ident {
                    all_used_names.insert(ast.get_atom(ident.id.sym(ast)));
                    continue;
                  }
                  if scope != top_level_scope_id {
                    all_used_names.insert(ast.get_atom(ident.id.sym(ast)));
                  }
                  let legacy = legacy.unwrap_or_else(|| ident.to_legacy(ast, &semantic));
                  concate_info.idents.push(legacy.clone());
                  binding_to_ref
                    .entry((legacy.id.sym.clone(), legacy.id.ctxt))
                    .or_default()
                    .push(legacy);
                }

                concate_info.all_used_names = all_used_names;
                concate_info.binding_to_ref = binding_to_ref;
                concate_info.has_ast = true;
                concate_info.source = Some(ReplaceSource::new(render_source.source.clone()));
                concate_info.internal_source = Some(render_source.source.clone());
                concate_info.runtime_requirements = codegen_res.runtime_requirements;
                concate_info.chunk_init_fragments = codegen_res
                  .data
                  .get::<ChunkInitFragments>()
                  .cloned()
                  .unwrap_or_default();
                concate_info
                  .chunk_init_fragments
                  .extend(codegen_res.chunk_init_fragments.clone());
                concate_info
                  .chunk_init_fragments
                  .extend(chunk_init_fragments);
                if let Some(CodeGenerationPublicPathAutoReplace(true)) =
                  codegen_res
                    .data
                    .get::<CodeGenerationPublicPathAutoReplace>()
                {
                  concate_info.public_path_auto_replacement = Some(true);
                }
                if codegen_res.data.contains::<URLStaticMode>() {
                  concate_info.static_url_replacement = true;
                }
                Ok((ModuleInfo::Concatenated(concate_info), None))
              }
            }
          },
        )
      }
    })
    .await;

    for m in map {
      let m = m.map_err(|e| rspack_error::error!(e.to_string()))?;
      let (m, external_fragments) = m.map_err(|e| rspack_error::error!(e.to_string()))?;
      if let Some((id, init_fragments)) = external_fragments {
        external_module_init_fragments.insert(id, init_fragments);
      }
      orig_concate_modules_map.insert(m.id(), m);
    }

    Ok(())
  }

  /**
  add __webpack_require__ call to current chunk at top level,
  if `from` is specified, the __webpack_require__ will be rendered
  as the `from` module renders.
  */
  pub(crate) fn add_require<'a>(
    m: ModuleIdentifier,
    from: Option<ModuleIdentifier>,
    symbol: Option<Atom>,
    all_used_names: &mut FxHashSet<Atom>,
    required: &'a mut IdentifierIndexMap<ExternalInterop>,
  ) -> &'a mut ExternalInterop {
    let require_info: &mut ExternalInterop = required.entry(m).or_insert(ExternalInterop {
      module: m,
      from_module: Default::default(),
      required_symbol: None,
      default_access: None,
      default_exported: None,
      namespace_object: None,
      namespace_object2: None,
      property_access: Default::default(),
    });

    if let Some(from) = from {
      require_info.from_module.insert(from);
    }

    if require_info.required_symbol.is_none()
      && let Some(symbol) = symbol
    {
      let new_name = if all_used_names.contains(&symbol) {
        let new_name = find_new_name(&symbol, all_used_names, &[]);
        all_used_names.insert(new_name.clone());
        new_name
      } else {
        all_used_names.insert(symbol.clone());
        symbol
      };

      require_info.required_symbol = Some(new_name);
    }

    require_info
  }

  fn resolve_re_export_star_from_unknown(
    module_id: ModuleIdentifier,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    side_effects_state_artifact: &SideEffectsStateArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
    collect_own_exports: bool,
    cache: &mut IdentifierMap<FxIndexSet<Either<Atom, ModuleIdentifier>>>,
  ) -> FxIndexSet<Either<Atom, ModuleIdentifier>> {
    // Memoize recursive calls to avoid redundant traversals of the same
    // module's export * chains (hot path: conn.is_active + dep.get_mode).
    if collect_own_exports && let Some(cached) = cache.get(&module_id) {
      return cached.clone();
    }

    let module = module_graph
      .module_by_identifier(&module_id)
      .expect("should have module");

    if module.as_external_module().is_some() {
      let result: FxIndexSet<_> = std::iter::once(Either::Right(module_id)).collect();
      if collect_own_exports {
        cache.insert(module_id, result.clone());
      }
      return result;
    }

    let mut exports = if collect_own_exports {
      let exports_info = exports_info_artifact.get_exports_info_data(&module_id);
      exports_info
        .exports()
        .iter()
        .filter(|(_, export_info)| matches!(export_info.provided(), Some(ExportProvided::Provided)))
        .map(|(name, _)| Either::Left(name.clone()))
        .collect()
    } else {
      FxIndexSet::default()
    };

    for dep in module.get_dependencies() {
      let Some(conn) = module_graph.connection_by_dependency_id(dep) else {
        continue;
      };

      if !conn.is_active(
        module_graph,
        None,
        module_graph_cache,
        side_effects_state_artifact,
        exports_info_artifact,
      ) {
        continue;
      }

      let dep = module_graph.dependency_by_id(dep);
      if let Some(dep) = dep.downcast_ref::<ESMExportImportedSpecifierDependency>()
        && dep.name.is_none()
      {
        let mode = dep.get_mode(
          module_graph,
          None,
          module_graph_cache,
          exports_info_artifact,
        );

        if matches!(mode, ExportMode::DynamicReexport(_)) {
          let ref_module = conn.module_identifier();
          exports.extend(Self::resolve_re_export_star_from_unknown(
            *ref_module,
            module_graph,
            module_graph_cache,
            side_effects_state_artifact,
            exports_info_artifact,
            true,
            cache,
          ));
        }
      }
    }

    if collect_own_exports {
      cache.insert(module_id, exports.clone());
    }

    exports
  }

  fn resolve_single_star_re_export_target(
    module_id: ModuleIdentifier,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    side_effects_state_artifact: &SideEffectsStateArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
    cache: &mut IdentifierMap<FxIndexSet<Either<Atom, ModuleIdentifier>>>,
  ) -> Option<ModuleIdentifier> {
    let mut star_target = None;
    for export in Self::resolve_re_export_star_from_unknown(
      module_id,
      module_graph,
      module_graph_cache,
      side_effects_state_artifact,
      exports_info_artifact,
      false,
      cache,
    ) {
      let Either::Right(module_id) = export else {
        continue;
      };

      match star_target {
        Some(existing) if existing != module_id => return None,
        Some(_) => {}
        None => star_target = Some(module_id),
      }
    }

    star_target
  }

  #[allow(clippy::too_many_arguments)]
  fn export_namespace_as_default(
    concate_modules_map: &mut IdentifierIndexMap<ModuleInfo>,
    entry_module: ModuleIdentifier,
    current_chunk: ChunkUkey,
    entry_chunk: ChunkUkey,
    link: &mut FxHashMap<ChunkUkey, ChunkLinkContext>,
    exports: &mut FxHashMap<ChunkUkey, ExportsContext>,
    required: &mut IdentifierIndexMap<ExternalInterop>,
    strict_current_chunk: bool,
    allow_rename: bool,
  ) {
    let module_info = concate_modules_map
      .get_mut(&entry_module)
      .expect("should have info");

    match module_info {
      ModuleInfo::Concatenated(info) => {
        let exported = Self::add_chunk_export(
          current_chunk,
          info
            .namespace_object_name
            .clone()
            .expect("should have namespace name"),
          "default".to_string().into(),
          exports,
          !allow_rename && (entry_chunk == current_chunk || strict_current_chunk),
        );

        if entry_chunk != current_chunk
          && let Some(exported) = exported
        {
          Self::add_chunk_re_export(
            entry_chunk,
            current_chunk,
            exported,
            "default".to_string().into(),
            exports,
            !allow_rename,
          );
        }
      }
      ModuleInfo::External(info) => {
        info.interop_default_access_used = true;

        let chunk_link = link.get_mut_unwrap(&entry_chunk);
        let required_info = Self::add_require(
          info.module,
          None,
          Some(info.name.clone().expect("should have required symbol")),
          &mut chunk_link.used_names,
          required,
        );

        required_info.default_access(&mut chunk_link.used_names);
        let symbol = required_info.default_exported(&mut chunk_link.used_names);
        Self::add_chunk_export(
          entry_chunk,
          symbol,
          "default".to_string().into(),
          exports,
          !allow_rename,
        );
      }
    }
  }

  #[allow(clippy::too_many_arguments)]
  fn link_entry_module_exports(
    &self,
    entry_module: ModuleIdentifier,
    current_chunk: ChunkUkey,
    entry_chunk: ChunkUkey,
    compilation: &Compilation,
    concate_modules_map: &mut IdentifierIndexMap<ModuleInfo>,
    required: &mut IdentifierIndexMap<ExternalInterop>,
    link: &mut FxHashMap<ChunkUkey, ChunkLinkContext>,
    needed_namespace_objects: &mut IdentifierIndexSet,
    entry_imports: &mut IdentifierIndexMap<FxHashMap<Atom, Atom>>,
    exports: &mut FxHashMap<ChunkUkey, ExportsContext>,
    escaped_identifiers: &FxHashMap<String, Vec<Atom>>,
    allow_rename: bool,
    filter_unused: bool,
    re_export_star_cache: &mut IdentifierMap<FxIndexSet<Either<Atom, ModuleIdentifier>>>,
  ) -> Vec<Diagnostic> {
    let mut errors = vec![];
    let context = &compilation.options.context;
    let module_graph = compilation.get_module_graph();

    let exports_info = compilation
      .exports_info_artifact
      .get_exports_info_data(&entry_module);

    // detect reexport star
    let mut star_re_exports_modules = IdentifierIndexSet::default();
    let keep_export_name =
      !allow_rename && (current_chunk == entry_chunk || self.strict_export_chunk(current_chunk));

    let mut entry_exports = exports_info
      .exports()
      .iter()
      .filter(|(_, export_info)| {
        !(matches!(export_info.provided(), Some(ExportProvided::NotProvided))
          || filter_unused && matches!(export_info.get_used(None), UsageState::Unused))
      })
      .map(|(name, _)| name.clone())
      .collect::<FxIndexSet<_>>();

    Self::resolve_re_export_star_from_unknown(
      entry_module,
      module_graph,
      &compilation.module_graph_cache_artifact,
      &compilation
        .build_module_graph_artifact
        .side_effects_state_artifact,
      &compilation.exports_info_artifact,
      // When filter_unused is true, own exports are already collected and filtered
      // by usage above — only collect `export *` targets here.
      // When filter_unused is false (entry modules), also collect own exports.
      !filter_unused,
      re_export_star_cache,
    )
    .iter()
    .for_each(|either| {
      match either {
        Either::Left(atom) => entry_exports.insert(atom.clone()),
        Either::Right(module_id) => star_re_exports_modules.insert(*module_id),
      };
    });

    let module = module_graph
      .module_by_identifier(&entry_module)
      .expect("should have module");

    let exports_type = module.get_exports_type(
      module_graph,
      &compilation.module_graph_cache_artifact,
      &compilation.exports_info_artifact,
      module.build_meta().strict_esm_module,
    );

    if matches!(exports_type, ExportsType::DefaultOnly) {
      Self::export_namespace_as_default(
        concate_modules_map,
        entry_module,
        current_chunk,
        entry_chunk,
        link,
        exports,
        required,
        self.strict_export_chunk(current_chunk),
        allow_rename,
      );
    } else {
      for name in entry_exports {
        if keep_export_name && name == "__esModule" {
          // no need to keep __esModule for esm output
          continue;
        }

        let chunk_link = link.get_mut_unwrap(&current_chunk);
        let Some(binding) = Self::get_binding(
          None,
          module_graph,
          &compilation.module_graph_cache_artifact,
          &compilation.exports_info_artifact,
          &entry_module,
          vec![name.clone()],
          concate_modules_map,
          needed_namespace_objects,
          false,
          false,
          module.build_meta().strict_esm_module,
          None,
          &mut Default::default(),
          required,
          &mut chunk_link.used_names,
        ) else {
          continue;
        };

        match binding {
          Ref::Symbol(symbol_binding) => {
            let ref_chunk = match Self::get_module_chunk(symbol_binding.module, compilation) {
              Ok(c) => c,
              Err(e) => {
                errors.push(e.into());
                continue;
              }
            };
            let ref_info = &mut concate_modules_map[&symbol_binding.module];

            match ref_info {
              ModuleInfo::External(_) => {
                // import the ref chunk
                entry_imports.entry(symbol_binding.module).or_default();

                let required_info = &mut required[&symbol_binding.module];

                let export_name = if let Some(id) = symbol_binding.ids.first() {
                  required_info.property_access(id, &mut chunk_link.used_names)
                } else if let Some(default_access) = &required_info.default_access
                  && default_access == &symbol_binding.symbol
                {
                  required_info.default_exported(&mut chunk_link.used_names)
                } else {
                  symbol_binding.symbol
                };

                let exported = Self::add_chunk_export(
                  entry_chunk,
                  export_name.clone(),
                  name.clone(),
                  exports,
                  !allow_rename,
                );

                if exported.is_none() && !allow_rename {
                  errors.push(
                    rspack_error::error!(
                      "Entry {entry_module} has conflict exports: {name} has already been exported"
                    )
                    .into(),
                  );
                }
              }
              ModuleInfo::Concatenated(_) => {
                let local_name = if symbol_binding.ids.is_empty() {
                  symbol_binding.render().into()
                } else {
                  let ref_chunk_link = link.get_mut_unwrap(&ref_chunk);
                  let new_name = find_new_name(&name, &ref_chunk_link.used_names, &[]);
                  ref_chunk_link.used_names.insert(new_name.clone());
                  ref_chunk_link
                    .decl_before_exports
                    .insert(format!("var {new_name} = {};\n", symbol_binding.render()));

                  new_name
                };

                let exported = Self::add_chunk_export(
                  ref_chunk,
                  local_name.clone(),
                  name.clone(),
                  exports,
                  !allow_rename
                    && (ref_chunk == entry_chunk || self.strict_export_chunk(ref_chunk)),
                );

                if exported.is_none()
                  && !allow_rename
                  && (ref_chunk == entry_chunk || self.strict_export_chunk(ref_chunk))
                {
                  errors.push(
                    rspack_error::error!(
                      "Entry {entry_module} has conflict exports: {name} has already been exported"
                    )
                    .into(),
                  );
                }

                if ref_chunk != entry_chunk
                  && let Some(exported) = exported
                {
                  Self::add_chunk_re_export(
                    entry_chunk,
                    ref_chunk,
                    exported.clone(),
                    name.clone(),
                    exports,
                    !allow_rename,
                  );
                }
              }
            }
          }
          Ref::Inline(inlined_value) => {
            let entry_chunk_link = link.get_mut_unwrap(&entry_chunk);
            let new_name = find_new_name(
              &name,
              &entry_chunk_link.used_names,
              &escaped_identifiers[&get_cached_readable_identifier(
                &entry_module,
                module_graph,
                &compilation.module_static_cache,
                context,
              )],
            );
            entry_chunk_link.used_names.insert(new_name.clone());
            entry_chunk_link
              .decl_before_exports
              .insert(format!("var {new_name} = {inlined_value};\n"));

            Self::add_chunk_export(
              entry_chunk,
              new_name.clone(),
              name.clone(),
              exports,
              !allow_rename,
            );
          }
        }
      }

      if matches!(exports_type, ExportsType::DefaultWithNamed) {
        Self::export_namespace_as_default(
          concate_modules_map,
          entry_module,
          current_chunk,
          entry_chunk,
          link,
          exports,
          required,
          self.strict_export_chunk(current_chunk),
          allow_rename,
        );
      }
    }

    let entry_chunk_link = link.get_mut_unwrap(&entry_chunk);

    for id in star_re_exports_modules {
      let external_module = module_graph
        .module_by_identifier(&id)
        .expect("should have module")
        .as_external_module()
        .expect("should be external module");
      entry_chunk_link
        .raw_star_exports
        .entry(external_module.get_request().primary().into())
        .or_default()
        .insert(START_EXPORTS.clone());
    }

    if concate_modules_map[&entry_module].is_external() {
      // execute
      Self::add_require(
        entry_module,
        None,
        None,
        &mut FxHashSet::default(),
        required,
      );
    }

    errors
  }

  fn link_imports_and_exports(
    &self,
    compilation: &Compilation,
    link: &mut FxHashMap<ChunkUkey, ChunkLinkContext>,
    runtime_chunks_exporting_require_via_runtime_module: &FxHashSet<ChunkUkey>,
    concate_modules_map: &mut IdentifierIndexMap<ModuleInfo>,
    needed_namespace_objects_by_ukey: &mut FxHashMap<ChunkUkey, IdentifierIndexSet>,
    escaped_identifiers: &FxHashMap<String, Vec<Atom>>,
  ) -> Vec<Diagnostic> {
    let mut errors = vec![];
    let context = &compilation.options.context;
    let module_graph = compilation.get_module_graph();

    // Cache for resolve_re_export_star_from_unknown to avoid redundant
    // traversals of the same module's export * chains across entry modules.
    let mut re_export_star_cache: IdentifierMap<FxIndexSet<Either<Atom, ModuleIdentifier>>> =
      IdentifierMap::default();

    // we don't modify exports and imports in chunk_link directly unless,
    // we re-borrow data from the chunk_link many times to avoid borrow
    // checker issue, so put chunk_link.exports, chunk_link.imports and
    // chunk_link.required ahead.
    let mut exports = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .keys()
      .map(|chunk| (*chunk, Default::default()))
      .collect::<FxHashMap<ChunkUkey, ExportsContext>>();
    let mut imports = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .keys()
      .map(|chunk| (*chunk, Default::default()))
      .collect::<FxHashMap<ChunkUkey, IdentifierIndexMap<FxHashMap<Atom, Atom>>>>();

    // const symbol = __webpack_require__(module);
    let mut required = FxHashMap::<ChunkUkey, IdentifierIndexMap<ExternalInterop>>::default();

    // link entry direct exports
    for (entry_name, entrypoint_ukey) in compilation.build_chunk_graph_artifact.entrypoints.iter() {
      let entrypoint = compilation
        .build_chunk_graph_artifact
        .chunk_group_by_ukey
        .expect_get(entrypoint_ukey);
      let entry_chunk_ukey = entrypoint.get_entrypoint_chunk();
      let needed_namespace = needed_namespace_objects_by_ukey
        .entry(entry_chunk_ukey)
        .or_default();

      let entry_imports = imports
        .get_mut(&entry_chunk_ukey)
        .unwrap_or_else(|| panic!("should set imports for chunk {entry_chunk_ukey:?}"));

      // the entry modules may be moved to other chunks, in that case, we should re-export the entry exports from
      // other chunks
      let entry_data = &compilation.entries[entry_name];

      for entry_module in entry_data
        .all_dependencies()
        .chain(compilation.global_entry.all_dependencies())
        .filter_map(|dep_id| module_graph.module_identifier_by_dependency_id(dep_id))
        .filter(|module| {
          compilation
            .code_generation_results
            .get_one(module)
            .get(&SourceType::JavaScript)
            .is_some()
        })
        .copied()
      {
        let entry_module_chunk = match Self::get_module_chunk(entry_module, compilation) {
          Ok(c) => c,
          Err(e) => {
            errors.push(e.into());
            continue;
          }
        };
        entry_imports.entry(entry_module).or_default();

        // NOTE: Similar hashbang and directives handling logic.
        // See rspack_plugin_rslib/src/plugin.rs render() for why this duplication is necessary.
        let hashbang = get_module_hashbang(module_graph, &entry_module);
        let directives = get_module_directives(module_graph, &entry_module);

        if let Some(hashbang) = &hashbang {
          let entry_chunk_link = link.get_mut_unwrap(&entry_chunk_ukey);
          entry_chunk_link.hashbang = Some(format!("{hashbang}\n"));
        }

        if let Some(directives) = directives {
          let entry_chunk_link = link.get_mut_unwrap(&entry_chunk_ukey);
          entry_chunk_link.directives = directives;
        }

        /*
        entry module sometimes are splitted to whatever chunk user needs,
        so the entry chunk maynot actually contains entry modules
         */
        let required = required.entry(entry_module_chunk).or_default();

        errors.extend(self.link_entry_module_exports(
          entry_module,
          entry_module_chunk,
          entry_chunk_ukey,
          compilation,
          concate_modules_map,
          required,
          link,
          needed_namespace,
          entry_imports,
          &mut exports,
          escaped_identifiers,
          false,
          false,
          &mut re_export_star_cache,
        ));
      }
    }

    if let Some(root) = &self.preserve_modules {
      let preserve_entry_modules = compilation
        .entries
        .values()
        .flat_map(|entry| entry.all_dependencies())
        .chain(compilation.global_entry.all_dependencies())
        .filter_map(|dep_id| module_graph.module_identifier_by_dependency_id(dep_id))
        .copied()
        .collect::<FxHashSet<_>>();

      let mut preserve_modules = module_graph.modules_keys().copied().collect::<Vec<_>>();
      preserve_modules.sort_unstable();

      for module_id in preserve_modules {
        if preserve_entry_modules.contains(&module_id) {
          continue;
        }

        if compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .get_module_chunks(module_id)
          .is_empty()
          || compilation
            .code_generation_results
            .get_one(&module_id)
            .get(&SourceType::JavaScript)
            .is_none()
        {
          continue;
        }

        let Some(normal_module) = module_graph
          .module_by_identifier(&module_id)
          .expect("should have module")
          .as_normal_module()
        else {
          continue;
        };

        let Some(abs_path) = normal_module
          .resource_resolved_data()
          .path()
          .map(|p| p.as_std_path())
        else {
          continue;
        };

        if !abs_path.starts_with(root) {
          continue;
        }

        let module_chunk = match Self::get_module_chunk(module_id, compilation) {
          Ok(c) => c,
          Err(e) => {
            errors.push(e.into());
            continue;
          }
        };

        let needed_namespace = needed_namespace_objects_by_ukey
          .entry(module_chunk)
          .or_default();
        let module_imports = imports
          .get_mut(&module_chunk)
          .unwrap_or_else(|| panic!("should set imports for chunk {module_chunk:?}"));
        module_imports.entry(module_id).or_default();

        let required = required.entry(module_chunk).or_default();

        errors.extend(self.link_entry_module_exports(
          module_id,
          module_chunk,
          module_chunk,
          compilation,
          concate_modules_map,
          required,
          link,
          needed_namespace,
          module_imports,
          &mut exports,
          escaped_identifiers,
          false,
          false,
          &mut re_export_star_cache,
        ));
      }
    }

    // Link dynamic import target exports.
    // Without facade chunks, exports go directly on the source chunk.
    // For multi-module chunks where exports may be renamed, we generate a namespace
    // object and record it in dyn_import_ns_map so that the dyn import template
    // can render `.then(m => m.<ns_name>)`.
    {
      let entry_chunk_ukey_set: FxHashSet<ChunkUkey> = compilation
        .build_chunk_graph_artifact
        .entrypoints
        .values()
        .map(|entrypoint_ukey| {
          compilation
            .build_chunk_graph_artifact
            .chunk_group_by_ukey
            .expect_get(entrypoint_ukey)
            .get_entrypoint_chunk()
        })
        .collect();

      let dyn_targets = {
        let all_dyn_targets = self.all_dyn_targets.borrow();
        let mut targets = all_dyn_targets.iter().copied().collect::<Vec<_>>();
        targets.sort();
        targets
      };

      for dyn_target in dyn_targets {
        // Skip orphan modules — they are not in any chunk (e.g. tree-shaken or worker entries)
        if compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .get_module_chunks(dyn_target)
          .is_empty()
        {
          continue;
        }
        let source_chunk = match Self::get_module_chunk(dyn_target, compilation) {
          Ok(c) => c,
          Err(e) => {
            errors.push(e.into());
            continue;
          }
        };
        if entry_chunk_ukey_set.contains(&source_chunk) {
          continue;
        }

        if compilation
          .code_generation_results
          .get_one(&dyn_target)
          .get(&SourceType::JavaScript)
          .is_none()
        {
          continue;
        }

        // No facade chunk — exports go directly on source_chunk
        let target_chunk = source_chunk;
        let is_strict = self.strict_export_chunk(target_chunk);
        let allow_rename = !is_strict;

        let needed_namespace = needed_namespace_objects_by_ukey
          .entry(target_chunk)
          .or_default();
        let target_imports = imports.entry(target_chunk).or_default();
        let required = required.entry(target_chunk).or_default();

        target_imports.entry(dyn_target).or_default();

        // Check if this module has a pre-assigned namespace name (set during optimize_chunks
        // for scope-hoisted modules in non-strict multi-module chunks).
        let ns_name = {
          let ns_map = self.dyn_import_ns_map.borrow();
          ns_map.get(&dyn_target).cloned()
        };

        if let Some(ns_name) = ns_name {
          // When a namespace object exists, consumers access this module via
          // `.then(m => m.<ns>)` — no need to export individual module exports.
          // Just export the namespace object itself.
          needed_namespace.insert(dyn_target);

          Self::add_chunk_export(
            target_chunk,
            ns_name.clone(),
            ns_name.clone(),
            &mut exports,
            false,
          );
        } else {
          // No namespace object — export individual module exports directly on the chunk.
          errors.extend(self.link_entry_module_exports(
            dyn_target,
            source_chunk,
            target_chunk,
            compilation,
            concate_modules_map,
            required,
            link,
            needed_namespace,
            target_imports,
            &mut exports,
            escaped_identifiers,
            allow_rename,
            true,
            &mut re_export_star_cache,
          ));
        }
      }
    }

    // calculate exports based on imports
    for (chunk, chunk_link) in link.iter_mut() {
      let mut refs = FxIndexMap::default();
      let needed_namespace_objects = needed_namespace_objects_by_ukey.entry(*chunk).or_default();
      let chunk_imports = imports.entry(*chunk).or_default();
      let required = required.entry(*chunk).or_default();
      let runtime_chunk = Self::get_runtime_chunk(*chunk, compilation);

      // check if needs runtime
      for m in chunk_link
        .decl_modules
        .iter()
        .chain(chunk_link.hoisted_modules.iter())
      {
        let info = &concate_modules_map[m];
        let runtime_requirements = info.get_runtime_requirements();
        if !runtime_requirements.is_empty() && runtime_chunk != *chunk {
          let runtime_template = compilation.runtime_template.create_runtime_code_template();
          let require_symbol: Atom = runtime_template
            .render_runtime_globals(&RuntimeGlobals::REQUIRE)
            .into();
          if !runtime_chunks_exporting_require_via_runtime_module.contains(&runtime_chunk) {
            Self::add_chunk_export(
              runtime_chunk,
              require_symbol.clone(),
              require_symbol.clone(),
              &mut exports,
              true,
            );
          }
          chunk_link.raw_import_stmts.insert(
            RawImportSource::Chunk(runtime_chunk),
            ImportSpec {
              atoms: std::iter::once((require_symbol.clone(), require_symbol.clone())).collect(),
              default_import: None,
              ns_import: None,
            },
          );
          break;
        }
      }

      // if one chunk has multiple modules that require the same
      // module, the first module require imported module, the
      // followings should not require the module again.
      //
      // ```js
      // const foo = __webpack_require__('foo')
      // foo; // access foo
      // foo; // access foo again, but no require call
      // ```
      for m in chunk_link.hoisted_modules.clone() {
        let module = module_graph
          .module_by_identifier(&m)
          .expect("should have module");

        // make sure all side-effect modules are rendered
        // eg.
        // import './foo.cjs'
        // should be rendered as __webpack_require__('./foo.cjs')
        for dep_id in module.get_dependencies() {
          let dep = module_graph.dependency_by_id(dep_id);

          let Some(conn) = module_graph.connection_by_dependency_id(dep_id) else {
            continue;
          };

          let ref_module = *conn.module_identifier();
          let ref_in_same_chunk = compilation
            .build_chunk_graph_artifact
            .chunk_graph
            .get_module_chunks(ref_module)
            .contains(chunk);

          // Fast path: for non-ESM imports to modules in the same chunk,
          // skip the expensive is_target_active check since chunk_imports
          // for same-chunk modules are unused.
          if !matches!(dep.dependency_type(), DependencyType::EsmImport) {
            if !ref_in_same_chunk
              && conn.is_target_active(
                module_graph,
                None,
                &compilation.module_graph_cache_artifact,
                &compilation
                  .build_module_graph_artifact
                  .side_effects_state_artifact,
                &compilation.exports_info_artifact,
              )
            {
              chunk_imports.entry(ref_module).or_default();
            }
            continue;
          }

          if !conn.is_target_active(
            module_graph,
            None,
            &compilation.module_graph_cache_artifact,
            &compilation
              .build_module_graph_artifact
              .side_effects_state_artifact,
            &compilation.exports_info_artifact,
          ) {
            continue;
          }

          let outgoing_module_info = &concate_modules_map[conn.module_identifier()];

          //ensure chunk
          chunk_imports.entry(ref_module).or_default();

          if outgoing_module_info.is_external() {
            if ChunkGraph::get_module_id(&compilation.module_ids_artifact, ref_module).is_none() {
              // if module don't contains id, it no need to be required
              // it's a hack for css-extract's css module
              continue;
            }

            Self::add_require(
              ref_module,
              Some(m),
              /*
              do not specify the symbol now, if it really needs to be used, it will be
              modified later by get_binding
              */
              None,
              &mut chunk_link.used_names,
              required,
            );
          }
        }

        let codegen_res = compilation.code_generation_results.get(&m, None);
        let concatenation_scope = codegen_res
          .concatenation_scope
          .as_ref()
          .expect("should have concatenation scope for scope hoisted module");

        for (ref_module, all_refs) in &concatenation_scope.refs {
          // import all atoms from ref_module
          for (ref_string, options) in all_refs.iter() {
            if refs.contains_key(ref_string) {
              continue;
            }

            let Some(binding) = Self::get_binding(
              Some(m),
              module_graph,
              &compilation.module_graph_cache_artifact,
              &compilation.exports_info_artifact,
              ref_module,
              options.ids.clone(),
              concate_modules_map,
              needed_namespace_objects,
              options.call,
              !options.direct_import,
              module.build_meta().strict_esm_module,
              options.asi_safe,
              &mut Default::default(),
              required,
              &mut chunk_link.used_names,
            ) else {
              continue;
            };

            refs.insert(
              ref_string
                .strip_suffix("._")
                .expect("should have prefix: '._'")
                .to_string(),
              binding,
            );
          }
        }
      }

      // deconflict imported symbol and required symbols
      // if symbol is from outside, we should deconflict them,
      // because we've only deconflicted local symbols before
      let mut ref_by_symbol =
        FxIndexMap::<(Atom, ModuleIdentifier), Vec<(String, SymbolRef)>>::default();
      let mut inline_refs = FxHashMap::<String, Ref>::default();

      refs
        .into_iter()
        .filter_map(|(ref_string, binding_ref)| match binding_ref {
          Ref::Symbol(symbol_ref) => Some((ref_string, symbol_ref)),
          Ref::Inline(inlined_string) => {
            inline_refs.insert(ref_string, Ref::Inline(inlined_string));
            None
          }
        })
        .for_each(|(ref_string, symbol_ref)| {
          ref_by_symbol
            .entry((symbol_ref.symbol.clone(), symbol_ref.module))
            .or_default()
            .push((ref_string, symbol_ref));
        });

      let mut refs = inline_refs;
      let all_used_names = &mut chunk_link.used_names;

      for ((symbol, m), mut all_refs) in ref_by_symbol {
        let ref_chunk = match Self::get_module_chunk(m, compilation) {
          Ok(c) => c,
          Err(e) => {
            errors.push(e.into());
            continue;
          }
        };
        let info = &concate_modules_map[&m];
        let from_external = matches!(info, ModuleInfo::External(_));
        let needs_import_chunk = ref_chunk != *chunk;

        if needs_import_chunk {
          // ensure we import this chunk
          chunk_imports.entry(m).or_default();
        }

        if needs_import_chunk && !from_external {
          let readable_identifier = get_cached_readable_identifier(
            &m,
            module_graph,
            &compilation.module_static_cache,
            context,
          );
          let (orig_symbol, local_symbol) = if all_used_names.contains(&symbol) {
            let new_symbol = find_new_name(
              &symbol,
              all_used_names,
              &escaped_identifiers[&readable_identifier],
            );
            all_used_names.insert(new_symbol.clone());

            for (_, cur_ref) in &mut all_refs {
              cur_ref.symbol = new_symbol.clone();
            }

            (symbol.clone(), new_symbol)
          } else {
            all_used_names.insert(symbol.clone());
            (symbol.clone(), symbol.clone())
          };

          // ref_chunk should ensure exporting the symbol
          let exported = Self::add_chunk_export(
            ref_chunk,
            orig_symbol.clone(),
            orig_symbol.clone(),
            &mut exports,
            false,
          )
          .expect("no strict export should always success");

          // import symbol from that chunk
          chunk_imports
            .entry(m)
            .or_default()
            .insert(exported, local_symbol);
        }

        for (ref_str, cur_ref) in all_refs {
          refs.insert(ref_str, Ref::Symbol(cur_ref));
        }
      }

      chunk_link.needed_namespace_objects = needed_namespace_objects.clone();
      chunk_link.refs = refs;

      // ensure imports external module
      for m in &chunk_link.decl_modules {
        let module = module_graph
          .module_by_identifier(m)
          .expect("should have module");
        for dep_id in module.get_dependencies() {
          let Some(conn) = module_graph.connection_by_dependency_id(dep_id) else {
            continue;
          };

          if !conn.is_target_active(
            module_graph,
            None,
            &compilation.module_graph_cache_artifact,
            &compilation
              .build_module_graph_artifact
              .side_effects_state_artifact,
            &compilation.exports_info_artifact,
          ) {
            continue;
          }

          let ref_module = conn.module_identifier();
          chunk_imports.entry(*ref_module).or_default();
        }
      }
    }

    /*
    optimize code style

    for example:

    ```js
    import 'external module'
    export * from 'external module'

    import { a } from 'external module'
    export { a }
    ```
    change to
    ```js
    export * from 'external module'

    export { a } from 'external module'
    ```
    */
    for chunk_link in link.values_mut() {
      let all_chunk_used_symbols = chunk_link
        .refs
        .iter()
        .filter_map(|(_, symbol_ref)| {
          if let Ref::Symbol(symbol_ref) = symbol_ref {
            Some(&symbol_ref.symbol)
          } else {
            None
          }
        })
        .collect::<FxHashSet<_>>();

      let all_chunk_exported_symbols = &exports[&chunk_link.chunk].exports;

      for (source, _) in &chunk_link.raw_star_exports {
        let key = (source.clone(), None);
        if let Some(import_spec) = chunk_link
          .raw_import_stmts
          .get(&RawImportSource::Source(key.clone()))
          && import_spec.atoms.is_empty()
          && import_spec.default_import.is_none()
          && import_spec.ns_import.is_none()
        {
          chunk_link
            .raw_import_stmts
            .swap_remove(&RawImportSource::Source(key));
        }
      }

      let mut removed_import_stmts = vec![];
      for (key, specifiers) in &chunk_link.raw_import_stmts {
        let RawImportSource::Source(key) = key else {
          continue;
        };

        if specifiers.atoms.is_empty() && specifiers.default_import.is_none() {
          // import 'externals'
          continue;
        }

        if key.1.is_some() {
          // TODO: optimize import attributes
          continue;
        }

        let locals = specifiers
          .atoms
          .iter()
          .map(|(imported, atom)| (atom.clone(), imported.clone()))
          .chain(
            specifiers
              .default_import
              .as_ref()
              .map(|atom| vec![(atom.clone(), "default".into())])
              .unwrap_or(vec![]),
          )
          .collect::<Vec<_>>();

        // check if all atoms are not used, and only used for export
        if locals.iter().all(|(local, _)| {
          // not accessed but exported
          !all_chunk_used_symbols.contains(local) && all_chunk_exported_symbols.contains_key(local)
        }) {
          // remove the import statement
          removed_import_stmts.push((key.clone(), locals));
        }
      }

      for (key, locals) in removed_import_stmts {
        chunk_link
          .raw_import_stmts
          .swap_remove(&RawImportSource::Source(key.clone()));

        // change it from normal export to re-export
        for (local, imported) in locals {
          let chunk_exports = exports
            .get_mut(&chunk_link.chunk)
            .expect("should have exports");

          // remove local from exported names
          let Some(export_names) = chunk_exports.exports.remove(&local) else {
            continue;
          };

          // add local to re-export
          for export_name in export_names {
            Self::add_re_export_from_request(
              chunk_link.chunk,
              key.0.clone(),
              imported.clone(),
              export_name,
              &mut exports,
            );
          }
        }
      }
    }

    // put result into chunk_link context
    for (chunk, exports) in exports {
      let link = link.get_mut(&chunk).expect("should have chunk");
      *link.exports_mut() = exports.exports;
      *link.re_exports_mut() = exports.re_exports;
    }
    for (chunk, imports) in imports {
      link.get_mut(&chunk).expect("should have chunk").imports = imports;
    }
    for (chunk, required) in required {
      link.get_mut(&chunk).expect("should have chunk").required = required;
    }

    errors
  }

  // the final name is the exact symbol in ref chunk
  #[allow(clippy::too_many_arguments)]
  fn get_binding(
    from: Option<ModuleIdentifier>,
    mg: &ModuleGraph,
    mg_cache: &ModuleGraphCacheArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
    info_id: &ModuleIdentifier,
    mut export_name: Vec<Atom>,
    module_to_info_map: &mut IdentifierIndexMap<ModuleInfo>,
    needed_namespace_objects: &mut IdentifierIndexSet,
    as_call: bool,
    call_context: bool,
    strict_esm_module: bool,
    asi_safe: Option<bool>,
    already_visited: &mut FxHashSet<ExportInfoHashKey>,
    required: &mut IdentifierIndexMap<ExternalInterop>,
    all_used_names: &mut FxHashSet<Atom>,
  ) -> Option<Ref> {
    let module = mg
      .module_by_identifier(info_id)
      .expect("should have module");
    let exports_type =
      module.get_exports_type(mg, mg_cache, exports_info_artifact, strict_esm_module);
    let info = &mut module_to_info_map[info_id];

    if export_name.is_empty() {
      match exports_type {
        ExportsType::DefaultOnly => {
          info.set_interop_namespace_object2_used(true);
          let symbol = match info {
            ModuleInfo::External(info) => {
              let required_info = Self::add_require(
                *info_id,
                from,
                Some(info.name.clone().expect("should have name")),
                all_used_names,
                required,
              );
              required_info.namespace2(all_used_names)
            }
            ModuleInfo::Concatenated(info) => info
              .interop_namespace_object2_name
              .clone()
              .expect("should already set interop namespace"),
          };

          return Some(Ref::Symbol(SymbolRef::new(
            *info_id,
            symbol,
            export_name.clone(),
            Arc::new(move |binding| binding.symbol.to_string()),
          )));
        }
        ExportsType::DefaultWithNamed => {
          info.set_interop_namespace_object_used(true);
          let symbol = match info {
            ModuleInfo::External(external_info) => {
              let required_info = Self::add_require(
                *info_id,
                from,
                Some(external_info.name.clone().expect("should have name")),
                all_used_names,
                required,
              );
              required_info.namespace(all_used_names)
            }
            ModuleInfo::Concatenated(info) => info
              .interop_namespace_object_name
              .clone()
              .expect("should already set interop namespace"),
          };

          return Some(Ref::Symbol(SymbolRef::new(
            *info_id,
            symbol,
            export_name.clone(),
            Arc::new(|binding| binding.symbol.to_string()),
          )));
        }
        _ => {}
      }
    } else {
      match exports_type {
        // normal case
        ExportsType::Namespace => {}
        ExportsType::DefaultWithNamed => match export_name.first().map(|atom| atom.as_str()) {
          Some("default") => {
            export_name = export_name[1..].to_vec();
          }
          Some("__esModule") => {
            return Some(es_module_binding());
          }
          _ => {}
        },
        ExportsType::DefaultOnly => {
          if export_name.first().map(|item| item.as_str()) == Some("__esModule") {
            return Some(es_module_binding());
          }

          let first_export_id = export_name.remove(0);
          if first_export_id != "default" {
            return Some(Ref::Inline(
              "/* non-default import from default-exporting module */undefined".into(),
            ));
          }
        }
        ExportsType::Dynamic => match export_name.first().map(|atom| atom.as_str()) {
          Some("default") => {
            // shadowing the previous immutable ref to avoid violating rustc borrow rules
            info.set_interop_default_access_used(true);
            let symbol = match info {
              ModuleInfo::External(info) => {
                let required_info = Self::add_require(
                  *info_id,
                  from,
                  Some(info.name.clone().expect("should have name")),
                  all_used_names,
                  required,
                );
                required_info.default_access(all_used_names)
              }
              ModuleInfo::Concatenated(info) => info
                .interop_default_access_name
                .clone()
                .expect("should already set interop namespace"),
            };

            export_name = export_name[1..].to_vec();

            return Some(Ref::Symbol(SymbolRef::new(
              *info_id,
              symbol,
              export_name.clone(),
              Arc::new(move |binding| {
                let default_access_name = &binding.symbol;
                let default_export = if as_call {
                  format!("{default_access_name}()")
                } else if let Some(true) = asi_safe {
                  format!("({default_access_name}())")
                } else if let Some(false) = asi_safe {
                  format!(";({default_access_name}())")
                } else {
                  format!("{default_access_name}.a")
                };

                let exports = format!("{default_export}{}", property_access(&binding.ids, 0));

                if !binding.ids.is_empty() && as_call && !call_context {
                  return if asi_safe.unwrap_or_default() {
                    format!("(0,{exports})")
                  } else if let Some(_asi_safe) = asi_safe {
                    format!(";(0,{exports})")
                  } else {
                    format!("/*#__PURE__*/Object({exports})")
                  };
                }

                exports
              }),
            )));
          }
          Some("__esModule") => {
            return Some(es_module_binding());
          }
          _ => {}
        },
      }
    }

    let exports_info = exports_info_artifact
      .get_prefetched_exports_info(info_id, PrefetchExportsInfoMode::Nested(&export_name));

    if export_name.is_empty() {
      let info = module_to_info_map.get_mut_unwrap(info_id);
      match info {
        ModuleInfo::Concatenated(info) => {
          needed_namespace_objects.insert(info.module);
          return Some(Ref::Symbol(SymbolRef::new(
            info.module,
            info
              .namespace_object_name
              .clone()
              .expect("should have namespace_object_name"),
            vec![],
            Arc::new(move |binding| normal_render(binding, as_call, call_context, asi_safe)),
          )));
        }
        ModuleInfo::External(info) => {
          return Some(Ref::Symbol(SymbolRef::new(
            info.module,
            Self::add_require(
              *info_id,
              from,
              Some(info.name.clone().expect("should have symbol")),
              all_used_names,
              required,
            )
            .required_symbol
            .clone()
            .expect("should have name"),
            vec![],
            Arc::new(move |binding| normal_render(binding, as_call, call_context, asi_safe)),
          )));
        }
      }
    }

    let export_info = exports_info.get_export_info_without_mut_module_graph(&export_name[0]);
    let export_info_hash_key = export_info.as_hash_key();

    if already_visited.contains(&export_info_hash_key) {
      return Some(Ref::Inline(
        "/* circular reexport */ Object(function x() { x() }())".into(),
      ));
    }

    already_visited.insert(export_info_hash_key);

    let info = &module_to_info_map[info_id];
    match info {
      ModuleInfo::Concatenated(info) => {
        let export_id = export_name.first().cloned();
        if matches!(export_info.provided(), Some(ExportProvided::NotProvided)) {
          let info = module_to_info_map
            .get_mut_unwrap(info_id)
            .as_concatenated_mut();
          needed_namespace_objects.insert(info.module);

          return Some(Ref::Symbol(SymbolRef::new(
            info.module,
            info
              .namespace_object_name
              .clone()
              .expect("should have namespace_object"),
            export_name.clone(),
            Arc::new(move |binding| normal_render(binding, as_call, call_context, asi_safe)),
          )));
        }

        let used_name = ExportsInfoGetter::get_used_name(
          GetUsedNameParam::WithNames(&exports_info),
          None,
          &export_name,
        );
        if let Some(ref export_id) = export_id
          && let Some(direct_export) = info.export_map.as_ref().and_then(|map| map.get(export_id))
        {
          if let Some(used_name) = used_name {
            match used_name {
              UsedName::Normal(used_name) => {
                let direct_export = Atom::new(direct_export.clone());
                let symbol = info
                  .get_internal_name(&direct_export)
                  .unwrap_or_else(|| panic!("should set internal name for {direct_export}"));

                return Some(Ref::Symbol(SymbolRef::new(
                  info.module,
                  symbol.clone(),
                  used_name[1..].to_vec(),
                  Arc::new(move |binding| normal_render(binding, as_call, call_context, asi_safe)),
                )));
              }
              UsedName::Inlined(inlined) => {
                return Some(Ref::Inline(inlined.render(&to_normal_comment(&format!(
                  "inlined export {}",
                  property_access(&export_name, 0)
                )))));
              }
            }
          } else {
            return Some(Ref::Inline("/* unused export */ undefined".into()));
          }
        }

        if let Some(ref export_id) = export_id
          && let Some(raw_export) = info
            .raw_export_map
            .as_ref()
            .and_then(|map| map.get(export_id))
        {
          let raw_export_symbol: Atom = raw_export.clone().into();
          let name = info
            .get_internal_name(&raw_export_symbol)
            .unwrap_or(&raw_export_symbol);
          return Some(Ref::Symbol(SymbolRef::new(
            info.module,
            name.clone(),
            export_name[1..].to_vec(),
            Arc::new(move |binding| normal_render(binding, as_call, call_context, asi_safe)),
          )));
        }

        let reexport = find_target(
          &export_info,
          mg,
          exports_info_artifact,
          Arc::new(|module: &ModuleIdentifier| module_to_info_map.contains_key(module)),
          &mut Default::default(),
        );
        match reexport {
          FindTargetResult::NoTarget => {}
          FindTargetResult::InvalidTarget(target) => {
            if let Some(export) = target.export {
              let exports_info = exports_info_artifact.get_prefetched_exports_info(
                &target.module,
                PrefetchExportsInfoMode::Nested(&export),
              );
              if let Some(UsedName::Inlined(inlined)) = ExportsInfoGetter::get_used_name(
                GetUsedNameParam::WithNames(&exports_info),
                None,
                &export,
              ) {
                return Some(Ref::Inline(inlined.inlined_value().render(
                  &to_normal_comment(&format!(
                    "inlined export {}",
                    property_access(&export_name, 0)
                  )),
                )));
              }
            }
            panic!(
              "Target module of reexport is not part of the concatenation (export '{:?}')",
              &export_id
            );
          }
          FindTargetResult::ValidTarget(reexport) => {
            if let Some(ref_info) = module_to_info_map.get(&reexport.module) {
              let build_meta = mg
                .module_by_identifier(info_id)
                .expect("should have module")
                .build_meta();

              return Self::get_binding(
                from,
                mg,
                mg_cache,
                exports_info_artifact,
                &ref_info.id(),
                if let Some(reexport_export) = reexport.export {
                  [reexport_export, export_name[1..].to_vec()].concat()
                } else {
                  export_name[1..].to_vec()
                },
                module_to_info_map,
                needed_namespace_objects,
                as_call,
                call_context,
                build_meta.strict_esm_module,
                asi_safe,
                already_visited,
                required,
                all_used_names,
              );
            }
          }
        }

        if info.namespace_export_symbol.is_some() {
          // That's how webpack write https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/ConcatenatedModule.js#L463-L471
          let used_name = ExportsInfoGetter::get_used_name(
            GetUsedNameParam::WithNames(&exports_info),
            None,
            &export_name,
          )
          .expect("should have export name");
          return Some(match used_name {
            UsedName::Normal(used_name) => Ref::Symbol(SymbolRef::new(
              info.module,
              info
                .namespace_object_name
                .clone()
                .expect("should have namespace_object_name"),
              used_name,
              Arc::new(move |binding| normal_render(binding, as_call, call_context, asi_safe)),
            )),
            // Inlined namespace export symbol is not possible for now but we compat it here
            UsedName::Inlined(inlined) => Ref::Inline(inlined.render("")),
          });
        }

        if let Some(UsedName::Inlined(inlined)) = used_name {
          return Some(Ref::Inline(inlined.render(&to_normal_comment(&format!(
            "inlined export {}",
            property_access(&export_name, 0)
          )))));
        }

        None
      }
      ModuleInfo::External(info) => {
        if let Some(used_name) = ExportsInfoGetter::get_used_name(
          GetUsedNameParam::WithNames(&exports_info),
          None,
          &export_name,
        ) {
          Some(match used_name {
            UsedName::Normal(used_name) => Ref::Symbol(SymbolRef::new(
              info.module,
              Self::add_require(
                *info_id,
                from,
                Some(info.name.clone().expect("should have symbol")),
                all_used_names,
                required,
              )
              .required_symbol
              .clone()
              .expect("should have symbol"),
              used_name,
              Arc::new(move |binding| normal_render(binding, as_call, call_context, asi_safe)),
            )),
            UsedName::Inlined(inlined) => Ref::Inline(inlined.render(&to_normal_comment(
              &format!("inlined export {}", property_access(&export_name, 0)),
            ))),
          })
        } else {
          Some(Ref::Inline("/* unused export */ undefined".into()))
        }
      }
    }
  }
}

fn es_module_binding() -> Ref {
  Ref::Inline("/* __esModule */true".into())
}

fn normal_render(
  binding: &SymbolRef,
  as_call: bool,
  call_context: bool,
  asi_safe: Option<bool>,
) -> String {
  let ids = &binding.ids;
  let reference = format!("{}{}", binding.symbol.as_ref(), property_access(ids, 0));

  if !ids.is_empty() && as_call && !call_context {
    return if asi_safe.unwrap_or_default() {
      format!("(0,{reference})")
    } else if let Some(_asi_safe) = asi_safe {
      format!(";(0,{reference})")
    } else {
      format!("/*#__PURE__*/Object({reference})")
    };
  }

  reference
}

#[cfg(test)]
mod tests {
  use rspack_core::{ChunkInitFragments, ChunkUkey, InitFragmentKey, ModuleIdentifier};
  use rspack_util::{
    atom::Atom,
    fx_hash::{FxHashMap, FxHashSet},
  };

  use crate::{EsmLibraryPlugin, chunk_link::RawImportSource};

  #[test]
  fn get_module_chunk_empty_chunks_returns_error() {
    let m = ModuleIdentifier::from("test_module");
    let chunks = FxHashSet::default();
    let result = EsmLibraryPlugin::validate_single_chunk(m, &chunks);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
      err_msg.contains("not in any chunk"),
      "expected 'not in any chunk', got: {err_msg}"
    );
  }

  #[test]
  fn get_module_chunk_multiple_chunks_returns_error() {
    let m = ModuleIdentifier::from("test_module");
    let mut chunks = FxHashSet::default();
    chunks.insert(ChunkUkey::new());
    chunks.insert(ChunkUkey::new());
    let result = EsmLibraryPlugin::validate_single_chunk(m, &chunks);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
      err_msg.contains("in multiple chunks"),
      "expected 'in multiple chunks', got: {err_msg}"
    );
  }

  #[test]
  fn get_module_chunk_single_chunk_returns_ok() {
    let m = ModuleIdentifier::from("test_module");
    let expected_chunk = ChunkUkey::new();
    let mut chunks = FxHashSet::default();
    chunks.insert(expected_chunk);
    let result = EsmLibraryPlugin::validate_single_chunk(m, &chunks);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected_chunk);
  }

  #[test]
  fn module_external_namespace_init_fragment_keeps_first_rendered_local() {
    let first_namespace_import = Atom::from("__rspack_external_0");
    let second_namespace_import = Atom::from("__rspack_external_1");
    let init_fragments: ChunkInitFragments = vec![
      Box::new(rspack_core::NormalInitFragment::new(
        "import * as __rspack_external_1 from \"../compiled/webpack-sources/index.js\";\n".into(),
        rspack_core::InitFragmentStage::StageESMImports,
        1,
        InitFragmentKey::ModuleExternal("../compiled/webpack-sources/index.js".into()),
        None,
      )),
      Box::new(rspack_core::NormalInitFragment::new(
        "import * as __rspack_external_0 from \"../compiled/webpack-sources/index.js\";\n".into(),
        rspack_core::InitFragmentStage::StageESMImports,
        0,
        InitFragmentKey::ModuleExternal("../compiled/webpack-sources/index.js".into()),
        None,
      )),
    ];
    let mut chunk_used_names = FxHashSet::default();
    let mut namespace_imports = FxHashMap::default();

    EsmLibraryPlugin::reserve_module_external_namespace_import_locals(
      &init_fragments,
      &mut chunk_used_names,
      Some(&mut namespace_imports),
    );

    assert_eq!(chunk_used_names.len(), 1);
    assert!(chunk_used_names.contains(&first_namespace_import));
    assert!(!chunk_used_names.contains(&second_namespace_import));
    assert_eq!(
      namespace_imports.get(&RawImportSource::Source((
        "../compiled/webpack-sources/index.js".into(),
        None,
      ))),
      Some(&first_namespace_import)
    );
  }

  #[test]
  fn module_external_namespace_init_fragment_prefers_decl_before_hoisted() {
    let decl_namespace_import = Atom::from("__rspack_external_decl");
    let hoisted_namespace_import = Atom::from("__rspack_external_hoisted");
    let decl_init_fragments: ChunkInitFragments =
      vec![Box::new(rspack_core::NormalInitFragment::new(
        "import * as __rspack_external_decl from \"../compiled/webpack-sources/index.js\";\n"
          .into(),
        rspack_core::InitFragmentStage::StageESMImports,
        0,
        InitFragmentKey::ModuleExternal("../compiled/webpack-sources/index.js".into()),
        None,
      ))];
    let hoisted_init_fragments: ChunkInitFragments =
      vec![Box::new(rspack_core::NormalInitFragment::new(
        "import * as __rspack_external_hoisted from \"../compiled/webpack-sources/index.js\";\n"
          .into(),
        rspack_core::InitFragmentStage::StageESMImports,
        0,
        InitFragmentKey::ModuleExternal("../compiled/webpack-sources/index.js".into()),
        None,
      ))];
    let mut chunk_used_names = FxHashSet::default();
    let mut namespace_imports = FxHashMap::default();

    EsmLibraryPlugin::reserve_module_external_namespace_import_locals(
      &decl_init_fragments,
      &mut chunk_used_names,
      Some(&mut namespace_imports),
    );
    EsmLibraryPlugin::reserve_module_external_namespace_import_locals(
      &hoisted_init_fragments,
      &mut chunk_used_names,
      Some(&mut namespace_imports),
    );

    assert_eq!(chunk_used_names.len(), 1);
    assert!(chunk_used_names.contains(&decl_namespace_import));
    assert!(!chunk_used_names.contains(&hoisted_namespace_import));
    assert_eq!(
      namespace_imports.get(&RawImportSource::Source((
        "../compiled/webpack-sources/index.js".into(),
        None,
      ))),
      Some(&decl_namespace_import)
    );
  }

  #[test]
  fn module_external_namespace_init_fragment_prefers_global_render_order() {
    let chunk_namespace_import = Atom::from("__rspack_external_chunk");
    let decl_namespace_import = Atom::from("__rspack_external_decl");
    let hoisted_namespace_import = Atom::from("__rspack_external_hoisted");
    let chunk_init_fragments: ChunkInitFragments =
      vec![Box::new(rspack_core::NormalInitFragment::new(
        "import * as __rspack_external_chunk from \"../compiled/webpack-sources/index.js\";\n"
          .into(),
        rspack_core::InitFragmentStage::StageESMImports,
        -1,
        InitFragmentKey::ModuleExternal("../compiled/webpack-sources/index.js".into()),
        None,
      ))];
    let decl_init_fragments: ChunkInitFragments =
      vec![Box::new(rspack_core::NormalInitFragment::new(
        "import * as __rspack_external_decl from \"../compiled/webpack-sources/index.js\";\n"
          .into(),
        rspack_core::InitFragmentStage::StageESMImports,
        1,
        InitFragmentKey::ModuleExternal("../compiled/webpack-sources/index.js".into()),
        None,
      ))];
    let hoisted_init_fragments: ChunkInitFragments =
      vec![Box::new(rspack_core::NormalInitFragment::new(
        "import * as __rspack_external_hoisted from \"../compiled/webpack-sources/index.js\";\n"
          .into(),
        rspack_core::InitFragmentStage::StageESMImports,
        0,
        InitFragmentKey::ModuleExternal("../compiled/webpack-sources/index.js".into()),
        None,
      ))];
    let mut chunk_used_names = FxHashSet::default();
    let mut namespace_imports = FxHashMap::default();

    EsmLibraryPlugin::reserve_module_external_namespace_import_locals_in_render_order(
      [
        &chunk_init_fragments,
        &decl_init_fragments,
        &hoisted_init_fragments,
      ],
      &mut chunk_used_names,
      Some(&mut namespace_imports),
    );

    assert_eq!(chunk_used_names.len(), 1);
    assert!(chunk_used_names.contains(&chunk_namespace_import));
    assert!(!chunk_used_names.contains(&decl_namespace_import));
    assert!(!chunk_used_names.contains(&hoisted_namespace_import));
    assert_eq!(
      namespace_imports.get(&RawImportSource::Source((
        "../compiled/webpack-sources/index.js".into(),
        None,
      ))),
      Some(&chunk_namespace_import)
    );
  }

  #[test]
  fn module_external_namespace_init_fragment_claims_chunk_top_level_name() {
    let module = ModuleIdentifier::from("test_module");
    let namespace_import = Atom::from("index_js_namespaceObject");
    let init_fragments: ChunkInitFragments = vec![Box::new(rspack_core::NormalInitFragment::new(
      "import * as index_js_namespaceObject from \"../compiled/webpack-sources/index.js\";\n"
        .into(),
      rspack_core::InitFragmentStage::StageESMImports,
      0,
      InitFragmentKey::ModuleExternal("../compiled/webpack-sources/index.js".into()),
      None,
    ))];
    let mut chunk_used_names = FxHashSet::default();
    let mut namespace_imports = FxHashMap::default();
    let mut required = Default::default();

    EsmLibraryPlugin::reserve_module_external_namespace_import_locals(
      &init_fragments,
      &mut chunk_used_names,
      Some(&mut namespace_imports),
    );

    let required_info = EsmLibraryPlugin::add_require(
      module,
      None,
      Some(namespace_import.clone()),
      &mut chunk_used_names,
      &mut required,
    );
    let required_symbol = required_info
      .required_symbol
      .as_ref()
      .expect("should allocate required symbol");

    assert_eq!(required_symbol.as_ref(), "index_js_namespaceObject_0");
    assert!(chunk_used_names.contains(&namespace_import));
    assert_eq!(
      namespace_imports.get(&RawImportSource::Source((
        "../compiled/webpack-sources/index.js".into(),
        None,
      ))),
      Some(&namespace_import)
    );
  }

  #[test]
  fn module_external_non_namespace_init_fragment_does_not_claim_chunk_top_level_name() {
    let init_fragments: ChunkInitFragments = vec![Box::new(rspack_core::NormalInitFragment::new(
      "import { createRequire as __rspack_createRequire } from \"node:module\";\nconst __rspack_createRequire_require = __rspack_createRequire(import.meta.url);\n"
        .into(),
      rspack_core::InitFragmentStage::StageESMImports,
      0,
      InitFragmentKey::ModuleExternal("node-commonjs".into()),
      None,
    ))];
    let mut chunk_used_names = FxHashSet::default();

    EsmLibraryPlugin::reserve_module_external_namespace_import_locals(
      &init_fragments,
      &mut chunk_used_names,
      None,
    );

    assert!(chunk_used_names.is_empty());
  }

  #[test]
  fn module_external_non_namespace_init_fragment_claims_top_level_decls() {
    let init_fragments: ChunkInitFragments = vec![Box::new(rspack_core::NormalInitFragment::new(
      "import { createRequire as __rspack_createRequire } from \"node:module\";\nconst __rspack_createRequire_require = __rspack_createRequire(import.meta.url);\n"
        .into(),
      rspack_core::InitFragmentStage::StageESMImports,
      0,
      InitFragmentKey::ModuleExternal("node-commonjs".into()),
      None,
    ))];
    let mut chunk_used_names = FxHashSet::default();

    EsmLibraryPlugin::reserve_module_external_top_level_decls(
      &init_fragments,
      &mut chunk_used_names,
    );

    assert!(chunk_used_names.contains(&Atom::from("__rspack_createRequire")));
    assert!(chunk_used_names.contains(&Atom::from("__rspack_createRequire_require")));
  }

  #[test]
  fn module_external_top_level_decls_keep_first_rendered_fragment() {
    let init_fragments: ChunkInitFragments = vec![
      Box::new(rspack_core::NormalInitFragment::new(
        "import { createRequire as __rspack_createRequire_1 } from \"node:module\";\nconst __rspack_createRequire_require_1 = __rspack_createRequire_1(import.meta.url);\n"
          .into(),
        rspack_core::InitFragmentStage::StageESMImports,
        1,
        InitFragmentKey::ModuleExternal("node-commonjs".into()),
        None,
      )),
      Box::new(rspack_core::NormalInitFragment::new(
        "import { createRequire as __rspack_createRequire_0 } from \"node:module\";\nconst __rspack_createRequire_require_0 = __rspack_createRequire_0(import.meta.url);\n"
          .into(),
        rspack_core::InitFragmentStage::StageESMImports,
        0,
        InitFragmentKey::ModuleExternal("node-commonjs".into()),
        None,
      )),
    ];
    let mut chunk_used_names = FxHashSet::default();

    EsmLibraryPlugin::reserve_module_external_top_level_decls(
      &init_fragments,
      &mut chunk_used_names,
    );

    assert!(chunk_used_names.contains(&Atom::from("__rspack_createRequire_0")));
    assert!(chunk_used_names.contains(&Atom::from("__rspack_createRequire_require_0")));
    assert!(!chunk_used_names.contains(&Atom::from("__rspack_createRequire_1")));
    assert!(!chunk_used_names.contains(&Atom::from("__rspack_createRequire_require_1")));
  }

  #[test]
  fn module_external_var_init_fragment_claims_top_level_decl() {
    let init_fragments: ChunkInitFragments = vec![Box::new(rspack_core::NormalInitFragment::new(
      "/* provided dependency */ var provided_identifier = __webpack_require__(\"./dep\");\n"
        .into(),
      rspack_core::InitFragmentStage::StageProvides,
      1,
      InitFragmentKey::ModuleExternal("provided provided_identifier".into()),
      None,
    ))];
    let mut chunk_used_names = FxHashSet::default();

    EsmLibraryPlugin::reserve_module_external_top_level_decls(
      &init_fragments,
      &mut chunk_used_names,
    );

    assert!(chunk_used_names.contains(&Atom::from("provided_identifier")));
  }

  #[test]
  fn external_candidate_name_does_not_claim_chunk_top_level_name() {
    let module = ModuleIdentifier::from("test_module");
    let mut candidate_used_names = FxHashSet::default();
    let mut chunk_used_names = FxHashSet::default();
    let mut required = Default::default();
    let escaped_identifiers =
      FxHashMap::from_iter([("./lib.js".to_string(), vec![Atom::from("lib")])]);

    let candidate = EsmLibraryPlugin::assign_external_candidate_name(
      "./lib.js",
      &mut candidate_used_names,
      &escaped_identifiers,
    );

    assert_eq!(candidate.as_ref(), "lib");
    assert!(candidate_used_names.contains(&candidate));
    assert!(!chunk_used_names.contains(&candidate));

    let required_info = EsmLibraryPlugin::add_require(
      module,
      None,
      Some(candidate.clone()),
      &mut chunk_used_names,
      &mut required,
    );

    assert_eq!(
      required_info
        .required_symbol
        .as_ref()
        .expect("should keep candidate"),
      &candidate
    );
    assert!(chunk_used_names.contains(&candidate));
  }
}
