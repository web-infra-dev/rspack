use std::{
  collections::{self},
  hash::BuildHasher,
  sync::{Arc, LazyLock},
};

use rayon::{iter::Either, prelude::*};
use rspack_collections::{IdentifierIndexMap, IdentifierIndexSet, IdentifierMap, UkeyMap, UkeySet};
use rspack_core::{
  BuildMetaDefaultObject, BuildMetaExportsType, ChunkGraph, ChunkInitFragments, ChunkUkey,
  CodeGenerationPublicPathAutoReplace, Compilation, ConcatenatedModuleIdent, DependencyType,
  ExportInfoHashKey, ExportMode, ExportProvided, ExportsInfoArtifact, ExportsInfoGetter,
  ExportsType, FindTargetResult, GetUsedNameParam, IdentCollector, ModuleGraph,
  ModuleGraphCacheArtifact, ModuleIdentifier, ModuleInfo, NAMESPACE_OBJECT_EXPORT, PathData,
  PrefetchExportsInfoMode, RuntimeGlobals, SourceType, URLStaticMode, UsageState, UsedName,
  UsedNameItem, escape_name, find_new_name, find_target, get_cached_readable_identifier,
  get_js_chunk_filename_template, get_module_directives, get_module_hashbang, property_access,
  property_name, reserved_names::RESERVED_NAMES, rspack_sources::ReplaceSource,
  split_readable_identifier, to_normal_comment,
};
use rspack_error::{Diagnostic, Error, Result};
use rspack_javascript_compiler::ast::Ast;
use rspack_plugin_javascript::{
  JsPlugin, RenderSource, dependency::ESMExportImportedSpecifierDependency,
  visitors::swc_visitor::resolver,
};
use rspack_util::{
  SpanExt,
  atom::Atom,
  fx_hash::{FxHashMap, FxHashSet, FxIndexMap, FxIndexSet, indexmap},
  swc::join_atom,
};
use swc_core::{
  common::{FileName, Spanned, SyntaxContext},
  ecma::{
    ast::{EsVersion, Program},
    parser::{EsSyntax, Syntax, parse_file_as_module},
  },
};

use crate::{
  EsmLibraryPlugin,
  chunk_link::{ChunkLinkContext, ExternalInterop, ReExportFrom, Ref, SymbolRef},
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

impl EsmLibraryPlugin {
  fn strict_export_chunk(&self, chunk: ChunkUkey) -> bool {
    self.strict_export_chunks.borrow().contains(&chunk)
  }

  fn add_chunk_export(
    chunk: ChunkUkey,
    local: Atom,
    exported: Atom,
    chunk_exports: &mut UkeyMap<ChunkUkey, ExportsContext>,
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
    chunk_exports: &mut UkeyMap<ChunkUkey, ExportsContext>,
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
    chunk_exports: &mut UkeyMap<ChunkUkey, ExportsContext>,
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

  pub(crate) async fn link(
    &self,
    compilation: &Compilation,
    diagnostics: &mut Vec<Diagnostic>,
  ) -> Result<()> {
    let module_graph = compilation.get_module_graph();

    // codegen uses self.concatenated_modules_map_for_codegen which has hold another Arc, so
    // it's safe to access concate_modules_map lock
    let mut concate_modules_map = self.concatenated_modules_map.write().await;

    // analyze every modules and collect identifiers to concate_modules_map
    self
      .analyze_module(compilation, &mut concate_modules_map)
      .await?;

    // initialize data for link chunks
    let mut link: UkeyMap<ChunkUkey, ChunkLinkContext> = compilation
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

    let (escaped_names, escaped_identifiers) = concate_modules_map
      .par_values()
      .map(|info| {
        let mut escaped_names: FxHashMap<String, String> = FxHashMap::default();
        let mut escaped_identifiers: FxHashMap<String, Vec<String>> = FxHashMap::default();
        let readable_identifier = get_cached_readable_identifier(
          &info.id(),
          module_graph,
          &compilation.module_static_cache,
          &compilation.options.context,
        );
        let splitted_readable_identifier: Vec<String> =
          split_readable_identifier(&readable_identifier);
        escaped_identifiers.insert(readable_identifier, splitted_readable_identifier);

        match info {
          ModuleInfo::Concatenated(info) => {
            for (id, _) in info.binding_to_ref.iter() {
              escaped_names.insert(id.0.to_string(), escape_name(id.0.as_str()));
            }

            if let Some(import_map) = &info.import_map {
              for ((source, _), imported_atoms) in import_map.iter() {
                escaped_identifiers
                  .insert(source.clone(), split_readable_identifier(source.as_str()));
                for atom in &imported_atoms.specifiers {
                  escaped_names.insert(atom.to_string(), escape_name(atom.as_str()));
                }
                if let Some(ns_import) = &imported_atoms.namespace {
                  escaped_names.insert(ns_import.to_string(), escape_name(ns_import.as_str()));
                }
              }
            }
          }
          ModuleInfo::External(_) => (),
        }
        (escaped_names, escaped_identifiers)
      })
      .reduce(
        || (FxHashMap::default(), FxHashMap::default()),
        |mut a, b| {
          a.0.extend(b.0);
          a.1.extend(b.1);
          a
        },
      );

    for chunk_link in link.values_mut() {
      self.deconflict_symbols(
        compilation,
        &mut concate_modules_map,
        chunk_link,
        &escaped_names,
        &escaped_identifiers,
      );
    }

    // link imported specifier with exported symbol
    let mut needed_namespace_objects_by_ukey = UkeyMap::default();
    diagnostics.extend(self.link_imports_and_exports(
      compilation,
      &mut link,
      &mut concate_modules_map,
      &mut needed_namespace_objects_by_ukey,
      &escaped_identifiers,
    ));

    let mut namespace_object_sources: IdentifierMap<String> = IdentifierMap::default();
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
              let mut binding = Self::get_binding(
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
              );

              if let Ref::Symbol(symbol_binding) = &mut binding
                && matches!(
                  concate_modules_map.get(&symbol_binding.module),
                  Some(ModuleInfo::External(_))
                )
              {
                chunk_link.imports.entry(symbol_binding.module).or_default();
              }

              ns_obj.push(format!(
                "\n  {}: {}",
                property_name(&used_name).expect("should have property_name"),
                runtime_template.returning_function(&binding.render(), "")
              ));
            }
          }
          // https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/optimize/ConcatenatedModule.js#L1539
          let name = namespace_name.expect("should have name_space_name");
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

          let module_info = concate_modules_map[module_info_id].as_concatenated_mut();

          namespace_object_sources.insert(
            *module_info_id,
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
            ),
          );

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
      let chunk = Self::get_module_chunk(module, compilation);
      let chunk_link = link.get_mut_unwrap(&chunk);
      chunk_link.namespace_object_sources.insert(module, source);
    }

    let mut links = self.links.borrow_mut();
    *links = link;
    Ok(())
  }

  pub fn get_module_chunk(m: ModuleIdentifier, compilation: &Compilation) -> ChunkUkey {
    let chunks = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_module_chunks(m);
    if chunks.is_empty() {
      panic!("module {m} is not in any chunk");
    }

    if chunks.len() > 1 {
      panic!("module {m} is in multiple chunks");
    }

    *chunks.iter().next().expect("at least one chunk")
  }

  fn deconflict_symbols(
    &self,
    compilation: &Compilation,
    concate_modules_map: &mut IdentifierIndexMap<ModuleInfo>,
    chunk_link: &mut ChunkLinkContext,
    escaped_names: &FxHashMap<String, String>,
    escaped_identifiers: &FxHashMap<String, Vec<String>>,
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
      all_used_names.extend(concate_info.all_used_names.clone());
    }

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
            let total_imported_atoms = chunk_link
              .raw_import_stmts
              .entry((source.clone(), attr.clone()))
              .or_default();

            if let Some(ns_import) = &imported_atoms.namespace {
              total_imported_atoms.ns_import = Some(ns_import.clone());
            }

            for atom in &imported_atoms.specifiers {
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
                    &escaped_names[atom.as_str()],
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
              &escaped_names[atom.as_str()],
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
          let name =
            Atom::from(namespace_export_symbol[NAMESPACE_OBJECT_EXPORT.len()..].to_string());
          all_used_names.insert(name.clone());
          concate_info
            .internal_names
            .insert(namespace_export_symbol.clone(), name.clone());
        }

        // Handle namespaceObjectName for concatenated type
        let namespace_object_name =
          if let Some(ref namespace_export_symbol) = concate_info.namespace_export_symbol {
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

        info.name = Some(find_new_name(
          "",
          &chunk_link.used_names,
          &escaped_identifiers[&readable_identifier],
        ));
      }
    }

    chunk_link.used_names = all_used_names;
  }

  async fn analyze_module(
    &self,
    compilation: &Compilation,
    orig_concate_modules_map: &mut IdentifierIndexMap<ModuleInfo>,
  ) -> Result<()> {
    let runtime_template = compilation.runtime_template.create_runtime_code_template();
    let mut outputs = UkeyMap::<ChunkUkey, String>::default();
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

      let chunk_ukey = Self::get_module_chunk(*m, compilation);
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
    let map = rspack_futures::scope::<_, _>(|token| {
      for (m, info) in concate_modules_map {
        // SAFETY: caller will poll the futures
        let s = unsafe { token.used((compilation, m, info, &runtime_template)) };
        s.spawn(
          async move |(compilation, id, info, runtime_template)| -> Result<ModuleInfo> {
            if compilation
              .build_chunk_graph_artifact
              .chunk_graph
              .get_module_chunks(m)
              .is_empty()
            {
              // orphan module
              return Ok(info);
            }

            let chunk_ukey = Self::get_module_chunk(m, compilation);

            let module_graph = compilation.get_module_graph();

            match info {
              rspack_core::ModuleInfo::External(mut external_module_info) => {
                // we use __webpack_require__.add({...}) to register modules
                external_module_info
                  .runtime_requirements
                  .insert(RuntimeGlobals::REQUIRE | RuntimeGlobals::MODULE_FACTORIES);
                Ok(ModuleInfo::External(external_module_info))
              }
              rspack_core::ModuleInfo::Concatenated(mut concate_info) => {
                let hooks = JsPlugin::get_compilation_hooks(compilation.id());
                let hooks = hooks.read().await;

                let codegen_res = compilation.code_generation_results.get(&id, None);
                let Some(js_source) = codegen_res.get(&SourceType::JavaScript) else {
                  return Ok(ModuleInfo::Concatenated(concate_info));
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
                let cm: Arc<swc_core::common::SourceMap> = Default::default();
                let readable_identifier = m.readable_identifier(&compilation.options.context);
                let fm = cm.new_source_file(
                  Arc::new(FileName::Custom(readable_identifier.clone().into_owned())),
                  render_source
                    .source
                    .source()
                    .into_string_lossy()
                    .into_owned(),
                );
                let mut errors = vec![];
                let module = match parse_file_as_module(
                  &fm,
                  Syntax::Es(EsSyntax {
                    jsx,
                    ..Default::default()
                  }),
                  EsVersion::EsNext,
                  None,
                  &mut errors,
                ) {
                  Ok(module) => module,
                  Err(err) => {
                    // return empty error as we already push error to compilation.diagnostics
                    return Err(Error::from_string(
                      Some(fm.src.clone().into_string()),
                      err.span().real_lo() as usize,
                      err.span().real_hi() as usize,
                      "JavaScript parse error:\n".to_string(),
                      err.kind().msg().to_string(),
                    ));
                  }
                };
                let mut ast = Ast::new(Program::Module(module), cm, None);

                let mut global_ctxt = SyntaxContext::empty();
                let mut module_ctxt = SyntaxContext::empty();
                let mut collector = IdentCollector::default();
                let mut all_used_names = FxHashSet::default();
                ast.transform(|program, context| {
                  global_ctxt = global_ctxt.apply_mark(context.unresolved_mark);
                  module_ctxt = module_ctxt.apply_mark(context.top_level_mark);
                  program.visit_mut_with(&mut resolver(
                    context.unresolved_mark,
                    context.top_level_mark,
                    false,
                  ));
                  program.visit_with(&mut collector);
                });

                let mut idents = vec![];
                for ident in collector.ids {
                  if ident.id.ctxt == global_ctxt {
                    all_used_names.insert(ident.id.sym.clone());
                  }
                  if ident.is_class_expr_with_ident {
                    all_used_names.insert(ident.id.sym.clone());
                    continue;
                  }
                  if ident.id.ctxt != module_ctxt {
                    all_used_names.insert(ident.id.sym.clone());
                  }
                  idents.push(ident);
                }

                let mut binding_to_ref: FxIndexMap<
                  (Atom, SyntaxContext),
                  Vec<ConcatenatedModuleIdent>,
                > = Default::default();

                for ident in &idents {
                  match binding_to_ref.entry((ident.id.sym.clone(), ident.id.ctxt)) {
                    indexmap::map::Entry::Occupied(mut occ) => {
                      occ.get_mut().push(ident.clone());
                    }
                    indexmap::map::Entry::Vacant(vac) => {
                      vac.insert(vec![ident.clone()]);
                    }
                  };
                }

                concate_info.global_scope_ident = idents
                  .iter()
                  .filter(|ident| ident.id.ctxt == global_ctxt)
                  .cloned()
                  .collect();
                concate_info.global_ctxt = global_ctxt;
                concate_info.module_ctxt = module_ctxt;
                concate_info.idents = idents;
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
                Ok(ModuleInfo::Concatenated(concate_info))
              }
            }
          },
        )
      }
    })
    .await;

    for m in map {
      let m = m.map_err(|e| rspack_error::error!(e.to_string()))?;
      let m = m.map_err(|e| rspack_error::error!(e.to_string()))?;
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
    exports_info_artifact: &ExportsInfoArtifact,
    collect_own_exports: bool,
  ) -> FxIndexSet<Either<Atom, ModuleIdentifier>> {
    let module = module_graph
      .module_by_identifier(&module_id)
      .expect("should have module");

    if module.as_external_module().is_some() {
      return std::iter::once(Either::Right(module_id)).collect();
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
          // collect all exports from ref module
          exports.extend(Self::resolve_re_export_star_from_unknown(
            *ref_module,
            module_graph,
            module_graph_cache,
            exports_info_artifact,
            true,
          ));
        }
      }
    }

    exports
  }

  #[allow(clippy::too_many_arguments)]
  fn export_namespace_as_default(
    concate_modules_map: &mut IdentifierIndexMap<ModuleInfo>,
    entry_module: ModuleIdentifier,
    current_chunk: ChunkUkey,
    entry_chunk: ChunkUkey,
    link: &mut UkeyMap<ChunkUkey, ChunkLinkContext>,
    exports: &mut UkeyMap<ChunkUkey, ExportsContext>,
    required: &mut IdentifierIndexMap<ExternalInterop>,
    strict_current_chunk: bool,
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
          entry_chunk == current_chunk || strict_current_chunk,
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
            true,
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
          true,
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
    link: &mut UkeyMap<ChunkUkey, ChunkLinkContext>,
    needed_namespace_objects: &mut IdentifierIndexSet,
    entry_imports: &mut IdentifierIndexMap<FxHashMap<Atom, Atom>>,
    exports: &mut UkeyMap<ChunkUkey, ExportsContext>,
    escaped_identifiers: &FxHashMap<String, Vec<String>>,
  ) -> Vec<Diagnostic> {
    let mut errors = vec![];
    let context = &compilation.options.context;
    let module_graph = compilation.get_module_graph();

    let exports_info = compilation
      .exports_info_artifact
      .get_exports_info_data(&entry_module);

    // detect reexport star
    let mut star_re_exports_modules = IdentifierIndexSet::default();
    let keep_export_name = current_chunk == entry_chunk || self.strict_export_chunk(current_chunk);

    let mut entry_exports = exports_info
      .exports()
      .iter()
      .filter(|(_, export_info)| {
        !matches!(export_info.provided(), Some(ExportProvided::NotProvided))
      })
      .map(|(name, _)| name.clone())
      .collect::<FxIndexSet<_>>();

    Self::resolve_re_export_star_from_unknown(
      entry_module,
      module_graph,
      &compilation.module_graph_cache_artifact,
      &compilation.exports_info_artifact,
      true,
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
      );
    } else {
      for name in entry_exports {
        if keep_export_name && name == "__esModule" {
          // no need to keep __esModule for esm output
          continue;
        }

        let chunk_link = link.get_mut_unwrap(&current_chunk);
        let binding = Self::get_binding(
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
        );

        match binding {
          Ref::Symbol(symbol_binding) => {
            let ref_chunk = Self::get_module_chunk(symbol_binding.module, compilation);
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
                  true,
                );

                if exported.is_none() {
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
                  ref_chunk == entry_chunk || self.strict_export_chunk(ref_chunk),
                );

                if exported.is_none()
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
                    true,
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

            Self::add_chunk_export(entry_chunk, new_name, name.clone(), exports, true);
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
    link: &mut UkeyMap<ChunkUkey, ChunkLinkContext>,
    concate_modules_map: &mut IdentifierIndexMap<ModuleInfo>,
    needed_namespace_objects_by_ukey: &mut UkeyMap<ChunkUkey, IdentifierIndexSet>,
    escaped_identifiers: &FxHashMap<String, Vec<String>>,
  ) -> Vec<Diagnostic> {
    let mut errors = vec![];
    let context = &compilation.options.context;
    let module_graph = compilation.get_module_graph();

    // we don't modify exports and imports in chunk_link directly unless,
    // we re-borrow data from the chunk_link many times to avoid borrow
    // checker issue, so put chunk_link.exports, chunk_link.imports and
    // chunk_link.required ahead.
    let mut exports = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .keys()
      .map(|chunk| (*chunk, Default::default()))
      .collect::<UkeyMap<ChunkUkey, ExportsContext>>();
    let mut imports = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .keys()
      .map(|chunk| (*chunk, Default::default()))
      .collect::<UkeyMap<ChunkUkey, IdentifierIndexMap<FxHashMap<Atom, Atom>>>>();

    // const symbol = __webpack_require__(module);
    let mut required = UkeyMap::<ChunkUkey, IdentifierIndexMap<ExternalInterop>>::default();

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
        let entry_module_chunk = Self::get_module_chunk(entry_module, compilation);
        entry_imports.entry(entry_module).or_default();

        // NOTE: Similar hashbang and directives handling logic.
        // See rspack_plugin_rslib/src/plugin.rs render() for why this duplication is necessary.
        let hashbang = get_module_hashbang(module_graph, &entry_module);
        let directives = get_module_directives(module_graph, &entry_module);

        if let Some(hashbang) = &hashbang {
          let entry_chunk_link = link.get_mut_unwrap(&entry_chunk_ukey);
          entry_chunk_link.init_fragments.insert(
            0,
            Box::new(rspack_core::NormalInitFragment::new(
              format!("{hashbang}\n"),
              rspack_core::InitFragmentStage::StageConstants,
              i32::MIN,
              rspack_core::InitFragmentKey::unique(),
              None,
            )),
          );
        }

        if let Some(directives) = directives {
          let entry_module_chunk_link = link.get_mut_unwrap(&entry_module_chunk);

          for (idx, directive) in directives.iter().enumerate() {
            let insert_pos = if hashbang.is_some() { 1 + idx } else { idx };
            entry_module_chunk_link.init_fragments.insert(
              insert_pos,
              Box::new(rspack_core::NormalInitFragment::new(
                format!("{directive}\n"),
                rspack_core::InitFragmentStage::StageConstants,
                i32::MIN + 1 + idx as i32,
                rspack_core::InitFragmentKey::unique(),
                None,
              )),
            );
          }
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
        ));
      }
    }

    // link facade chunk (dyn import namespace) exports
    // Similar to entry chunks, facade chunks need all exports registered with exact names.
    // This handles star re-exports and ensures completeness beyond what dyn_refs provides.
    //
    // For modules with a facade chunk (empty, only re-exports):
    //   current_chunk = source chunk (where the module actually lives)
    //   entry_chunk = facade chunk (where exports are registered)
    // For modules without a facade (single-module chunks):
    //   current_chunk = entry_chunk = the module's chunk
    {
      let entry_chunk_ukey_set: UkeySet<ChunkUkey> = compilation
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

      let facade_map = self.dyn_import_facade_chunks.borrow();
      let facade_modules = {
        let all_dyn_targets = self.all_dyn_targets.borrow();
        let mut facade_modules = all_dyn_targets.iter().copied().collect::<Vec<_>>();
        facade_modules.sort();
        facade_modules
      };

      for facade_module in facade_modules {
        let source_chunk = Self::get_module_chunk(facade_module, compilation);
        if entry_chunk_ukey_set.contains(&source_chunk) {
          continue;
        }

        if compilation
          .code_generation_results
          .get_one(&facade_module)
          .get(&SourceType::JavaScript)
          .is_none()
        {
          continue;
        }

        // If there's a facade chunk, exports go to the facade; module code is in source_chunk.
        // Otherwise, both are the same (single-module chunk).
        let entry_chunk = facade_map
          .get(&facade_module)
          .copied()
          .unwrap_or(source_chunk);

        let needed_namespace = needed_namespace_objects_by_ukey
          .entry(entry_chunk)
          .or_default();
        let facade_imports = imports.entry(entry_chunk).or_default();
        let required = required.entry(entry_chunk).or_default();

        facade_imports.entry(facade_module).or_default();

        errors.extend(self.link_entry_module_exports(
          facade_module,
          source_chunk,
          entry_chunk,
          compilation,
          concate_modules_map,
          required,
          link,
          needed_namespace,
          facade_imports,
          &mut exports,
          escaped_identifiers,
        ));
      }
    }

    // calculate exports based on imports
    for (chunk, chunk_link) in link.iter_mut() {
      let mut refs = FxIndexMap::default();
      let mut dyn_refs = FxHashMap::default();
      let needed_namespace_objects = needed_namespace_objects_by_ukey.entry(*chunk).or_default();
      let chunk_imports = imports.entry(*chunk).or_default();
      let required = required.entry(*chunk).or_default();
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
          if !conn.is_target_active(
            module_graph,
            None,
            &compilation.module_graph_cache_artifact,
            &compilation.exports_info_artifact,
          ) {
            continue;
          }

          let outgoing_module_info = &concate_modules_map[conn.module_identifier()];
          let ref_module = *conn.module_identifier();

          //ensure chunk
          chunk_imports.entry(ref_module).or_default();

          if !matches!(dep.dependency_type(), DependencyType::EsmImport) {
            continue;
          }

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

            let binding = Self::get_binding(
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
            );

            refs.insert(
              ref_string
                .strip_suffix("._")
                .expect("should have prefix: '._'")
                .to_string(),
              binding,
            );
          }
        }

        for (ref_module, all_refs) in &concatenation_scope.dyn_refs {
          let ref_chunk = Self::get_module_chunk(*ref_module, compilation);
          let from_other_chunk = ref_chunk != *chunk;

          for (ref_string, ref_atom) in all_refs.iter() {
            if dyn_refs.contains_key(ref_string) {
              continue;
            }

            let mut binding = Self::get_binding(
              None,
              module_graph,
              &compilation.module_graph_cache_artifact,
              &compilation.exports_info_artifact,
              ref_module,
              vec![ref_atom.clone()],
              concate_modules_map,
              needed_namespace_objects,
              false,
              false,
              module.build_meta().strict_esm_module,
              Some(false),
              &mut Default::default(),
              required,
              &mut chunk_link.used_names,
            );

            if let Ref::Symbol(symbol_binding) = &mut binding {
              let module_id = symbol_binding.module;
              let ref_module_chunk = Self::get_module_chunk(module_id, compilation);
              let ref_external = concate_modules_map[ref_module].is_external();

              if from_other_chunk && !ref_external {
                let exported = Self::add_chunk_export(
                  ref_module_chunk,
                  symbol_binding.symbol.clone(),
                  symbol_binding.symbol.clone(),
                  &mut exports,
                  self.strict_export_chunk(ref_module_chunk),
                );

                let Some(mut exported) = exported else {
                  if self.strict_export_chunk(ref_module_chunk) {
                    errors.push(
                      rspack_error::error!(
                        "Dynamic import module {ref_module} has conflict exports: {} has already been exported",
                        symbol_binding.symbol
                      )
                      .into(),
                    );
                  }
                  continue;
                };

                if ref_module_chunk != ref_chunk {
                  // special case
                  // const { foo, bar } = await import('./re-exports')
                  // there is a chance that foo is from another chunk, and bar is from re-exports chunk
                  // so should make sure foo is from another chunk
                  let Some(reexported) = Self::add_chunk_re_export(
                    ref_chunk,
                    ref_module_chunk,
                    exported.clone(),
                    exported.clone(),
                    &mut exports,
                    self.strict_export_chunk(ref_chunk),
                  ) else {
                    if self.strict_export_chunk(ref_chunk) {
                      errors.push(
                        rspack_error::error!(
                          "Dynamic import module {ref_module} has conflict re-exports: {} has already been exported",
                          exported
                        )
                        .into(),
                      );
                    }
                    continue;
                  };

                  exported = reexported.clone();
                }

                symbol_binding.symbol = exported;
              }
            }

            dyn_refs.insert(ref_string.clone(), (!from_other_chunk, binding));
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
        let ref_chunk = Self::get_module_chunk(m, compilation);
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
      chunk_link.dyn_refs = dyn_refs;

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
        .chain(
          chunk_link
            .dyn_refs
            .iter()
            .filter_map(|(_, (_, symbol_ref))| {
              if let Ref::Symbol(symbol_ref) = symbol_ref {
                Some(&symbol_ref.symbol)
              } else {
                None
              }
            }),
        )
        .collect::<FxHashSet<_>>();

      let all_chunk_exported_symbols = &exports[&chunk_link.chunk].exports;

      for (source, _) in &chunk_link.raw_star_exports {
        let key = (source.clone(), None);
        if let Some(import_spec) = chunk_link.raw_import_stmts.get(&key)
          && import_spec.atoms.is_empty()
          && import_spec.default_import.is_none()
          && import_spec.ns_import.is_none()
        {
          chunk_link.raw_import_stmts.swap_remove(&key);
        }
      }

      let mut removed_import_stmts = vec![];
      for (key, specifiers) in &chunk_link.raw_import_stmts {
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
        chunk_link.raw_import_stmts.swap_remove(&key);

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
  ) -> Ref {
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

          return Ref::Symbol(SymbolRef::new(
            *info_id,
            symbol,
            export_name.clone(),
            Arc::new(move |binding| binding.symbol.to_string()),
          ));
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

          return Ref::Symbol(SymbolRef::new(
            *info_id,
            symbol,
            export_name.clone(),
            Arc::new(|binding| binding.symbol.to_string()),
          ));
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
            return es_module_binding();
          }
          _ => {}
        },
        ExportsType::DefaultOnly => {
          if export_name.first().map(|item| item.as_str()) == Some("__esModule") {
            return es_module_binding();
          }

          let first_export_id = export_name.remove(0);
          if first_export_id != "default" {
            return Ref::Inline(
              "/* non-default import from default-exporting module */undefined".into(),
            );
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

            return Ref::Symbol(SymbolRef::new(
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
            ));
          }
          Some("__esModule") => {
            return es_module_binding();
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
          return Ref::Symbol(SymbolRef::new(
            info.module,
            info
              .namespace_object_name
              .clone()
              .expect("should have namespace_object_name"),
            vec![],
            Arc::new(move |binding| normal_render(binding, as_call, call_context, asi_safe)),
          ));
        }
        ModuleInfo::External(info) => {
          return Ref::Symbol(SymbolRef::new(
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
          ));
        }
      }
    }

    let export_info = exports_info.get_export_info_without_mut_module_graph(&export_name[0]);
    let export_info_hash_key = export_info.as_hash_key();

    if already_visited.contains(&export_info_hash_key) {
      return Ref::Inline("/* circular reexport */ Object(function x() { x() }())".into());
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

          return Ref::Symbol(SymbolRef::new(
            info.module,
            info
              .namespace_object_name
              .clone()
              .expect("should have namespace_object"),
            export_name.clone(),
            Arc::new(move |binding| normal_render(binding, as_call, call_context, asi_safe)),
          ));
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

                return Ref::Symbol(SymbolRef::new(
                  info.module,
                  symbol.clone(),
                  used_name[1..].to_vec(),
                  Arc::new(move |binding| normal_render(binding, as_call, call_context, asi_safe)),
                ));
              }
              UsedName::Inlined(inlined) => {
                return Ref::Inline(inlined.render(&to_normal_comment(&format!(
                  "inlined export {}",
                  property_access(&export_name, 0)
                ))));
              }
            }
          } else {
            return Ref::Inline("/* unused export */ undefined".into());
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
          return Ref::Symbol(SymbolRef::new(
            info.module,
            name.clone(),
            export_name[1..].to_vec(),
            Arc::new(move |binding| normal_render(binding, as_call, call_context, asi_safe)),
          ));
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
                return Ref::Inline(inlined.inlined_value().render(&to_normal_comment(&format!(
                  "inlined export {}",
                  property_access(&export_name, 0)
                ))));
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
          return match used_name {
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
          };
        }

        if let Some(UsedName::Inlined(inlined)) = used_name {
          return Ref::Inline(inlined.render(&to_normal_comment(&format!(
            "inlined export {}",
            property_access(&export_name, 0)
          ))));
        }

        panic!(
          "Cannot get final name for export '{}' of module '{}'",
          join_atom(export_name.iter(), "."),
          module.identifier()
        );
      }
      ModuleInfo::External(info) => {
        if let Some(used_name) = ExportsInfoGetter::get_used_name(
          GetUsedNameParam::WithNames(&exports_info),
          None,
          &export_name,
        ) {
          match used_name {
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
          }
        } else {
          Ref::Inline("/* unused export */ undefined".into())
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
