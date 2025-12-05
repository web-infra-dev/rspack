use std::{
  collections::{self},
  hash::BuildHasher,
  sync::{Arc, LazyLock},
};

use rayon::prelude::*;
use rspack_collections::{IdentifierIndexMap, IdentifierIndexSet, IdentifierMap, UkeyMap};
use rspack_core::{
  BuildMetaDefaultObject, BuildMetaExportsType, ChunkGraph, ChunkInitFragments, ChunkUkey,
  CodeGenerationPublicPathAutoReplace, Compilation, ConcatenatedModuleIdent, DependencyType,
  ExportMode, ExportModeNormalReexport, ExportProvided, ExportsInfoGetter, ExportsType,
  ExternalModule, FindTargetResult, GetUsedNameParam, IdentCollector, InitFragmentKey,
  InitFragmentStage, MaybeDynamicTargetExportInfoHashKey, ModuleGraph, ModuleGraphCacheArtifact,
  ModuleIdentifier, ModuleInfo, NAMESPACE_OBJECT_EXPORT, NormalInitFragment, NormalReexportItem,
  PathData, PrefetchExportsInfoMode, RuntimeGlobals, SourceType, URLStaticMode, UsageState,
  UsedName, UsedNameItem, escape_name, find_new_name, get_cached_readable_identifier,
  get_js_chunk_filename_template, get_target, property_access, property_name,
  reserved_names::RESERVED_NAMES, rspack_sources::ReplaceSource, split_readable_identifier,
  to_normal_comment,
};
use rspack_error::{Diagnostic, Result};
use rspack_javascript_compiler::ast::Ast;
use rspack_plugin_javascript::{
  JS_DEFAULT_KEYWORD, JsPlugin, RenderSource, dependency::ESMExportImportedSpecifierDependency,
  visitors::swc_visitor::resolver,
};
use rspack_util::{
  atom::Atom,
  fx_hash::{FxHashMap, FxHashSet, FxIndexMap, FxIndexSet, indexmap},
  swc::join_atom,
};
use swc_core::{
  common::{FileName, SyntaxContext},
  ecma::{
    ast::{EsVersion, Program},
    parser::{Syntax, parse_file_as_module},
  },
};

use crate::{
  EsmLibraryPlugin,
  chunk_link::{ChunkLinkContext, ExternalInterop, Ref, SymbolRef},
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

#[derive(Default)]
struct ExportsContext {
  pub exports: FxHashMap<Atom, FxIndexSet<Atom>>,
  pub exported_symbols: FxHashMap<Atom, Atom>,
}

impl EsmLibraryPlugin {
  fn chunk_symbol_for_export<'a>(
    chunk: ChunkUkey,
    exported: &Atom,
    chunk_exports: &'a mut UkeyMap<ChunkUkey, ExportsContext>,
  ) -> Option<&'a Atom> {
    let ctx = chunk_exports.get_mut_unwrap(&chunk);
    ctx.exported_symbols.get(exported)
  }

  fn add_chunk_export(
    chunk: ChunkUkey,
    local: Atom,
    exported: Atom,
    chunk_exports: &mut UkeyMap<ChunkUkey, ExportsContext>,
    strict_exports: bool,
  ) -> Option<Atom> {
    {
      let ctx = chunk_exports.get_mut_unwrap(&chunk);

      // check if we've already exported this local symbol
      if let Some(already_exported) = ctx.exports.get_mut(&local)
        && !already_exported.is_empty()
      {
        if strict_exports {
          if let Some(exported_local) = ctx.exported_symbols.get(&exported)
            && exported_local != &local
          {
            // already exported the symbol and not the same local symbol
            return None;
          }
          already_exported.insert(exported.clone());
          ctx.exported_symbols.insert(exported.clone(), local.clone());
          return ctx.exported_symbols.get(&exported).cloned();
        } else {
          // not strict exports, we can export whatever we like
          return Some(
            already_exported
              .get(&exported)
              .unwrap_or(already_exported.iter().next().expect("should have export"))
              .clone(),
          );
        }
      }
    }

    let ctx = chunk_exports.get_mut_unwrap(&chunk);

    // we've not exported this local symbol, check if we've already exported this symbol
    if ctx.exported_symbols.contains_key(&exported) {
      // the name is already exported and we know the exported_local is not the same
      if strict_exports {
        return None;
      }

      let already_exported_names = ctx.exports.entry(local.clone()).or_default();

      // we find another name to export this symbol
      let mut idx = 0;
      let mut new_export = Atom::new(format!("{exported}_{idx}"));
      while ctx.exported_symbols.contains_key(&new_export) {
        idx += 1;
        new_export = format!("{exported}_{idx}").into();
      }

      ctx
        .exported_symbols
        .insert(new_export.clone(), local.clone());
      already_exported_names.insert(new_export.clone());
      already_exported_names.get(&new_export).cloned()
    } else {
      let already_exported_names = ctx.exports.entry(local.clone()).or_default();
      ctx.exported_symbols.insert(exported.clone(), local.clone());
      already_exported_names.insert(exported.clone());
      already_exported_names.get(&exported).cloned()
    }
  }

  pub(crate) async fn link(&self, compilation: &mut Compilation) -> Result<()> {
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
      .chunk_by_ukey
      .keys()
      .map(|ukey| {
        let modules = compilation.chunk_graph.get_chunk_modules_identifier(ukey);

        let mut decl_modules = IdentifierIndexSet::default();
        let mut hoisted_modules = IdentifierIndexSet::default();

        for m in modules.iter() {
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
          &module_graph,
          &compilation.module_static_cache_artifact,
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
                for atom in imported_atoms {
                  escaped_names.insert(atom.to_string(), escape_name(atom.as_str()));
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
    compilation.extend_diagnostics(self.link_imports_and_exports(
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
          let exports_info = module_graph.get_exports_info(module_info_id);
          for export_info in exports_info.as_data(&module_graph).exports().values() {
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
                &compilation.get_module_graph(),
                &compilation.module_graph_cache_artifact,
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
                compilation
                  .runtime_template
                  .returning_function(&binding.render(), "")
              ));
            }
          }
          // https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/optimize/ConcatenatedModule.js#L1539
          let name = namespace_name.expect("should have name_space_name");
          let define_getters = if !ns_obj.is_empty() {
            format!(
              "{}({}, {{ {} }});\n",
              compilation
                .runtime_template
                .render_runtime_globals(&RuntimeGlobals::DEFINE_PROPERTY_GETTERS),
              name,
              ns_obj.join(",")
            )
          } else {
            String::new()
          };

          let module_info = concate_modules_map[module_info_id].as_concatenated_mut();

          if !ns_obj.is_empty() {
            module_info
              .runtime_requirements
              .insert(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);
          }

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
              compilation
                .runtime_template
                .render_runtime_globals(&RuntimeGlobals::MAKE_NAMESPACE_OBJECT),
              name,
              define_getters
            ),
          );

          module_info
            .runtime_requirements
            .insert(RuntimeGlobals::MAKE_NAMESPACE_OBJECT);
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

  pub fn is_orphan(m: ModuleIdentifier, compilation: &Compilation) -> bool {
    compilation.chunk_graph.get_module_chunks(m).is_empty()
  }

  pub fn get_module_chunk(m: ModuleIdentifier, compilation: &Compilation) -> ChunkUkey {
    let chunks = compilation.chunk_graph.get_module_chunks(m);
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
      let readable_identifier = get_cached_readable_identifier(
        id,
        &module_graph,
        &compilation.module_static_cache_artifact,
        context,
      );

      if let ModuleInfo::Concatenated(concate_info) = info {
        let mut internal_names = FxHashMap::default();

        // registered import map
        if let Some(import_map) = &concate_info.import_map {
          for ((source, attr), imported_atoms) in import_map.iter() {
            let total_imported_atoms = chunk_link
              .raw_import_stmts
              .entry((source.clone(), attr.clone()))
              .or_default();

            for atom in imported_atoms {
              // already import this symbol
              if let Some(internal_atom) = total_imported_atoms.atoms.get(atom) {
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
          &module_graph,
          &compilation.module_static_cache_artifact,
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
    let mut outputs = UkeyMap::<ChunkUkey, String>::default();
    let concate_modules_map = orig_concate_modules_map.clone();
    for m in concate_modules_map.keys() {
      if compilation.chunk_graph.get_module_chunks(*m).is_empty() {
        // orphan module
        continue;
      }

      let chunk_ukey = Self::get_module_chunk(*m, compilation);
      let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
      let filename_template = get_js_chunk_filename_template(
        chunk,
        &compilation.options.output,
        &compilation.chunk_group_by_ukey,
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

    let map = rspack_futures::scope::<_, _>(|token| {
      for (m, info) in concate_modules_map {
        if compilation.chunk_graph.get_module_chunks(m).is_empty() {
          // orphan module
          continue;
        }
        let chunk_ukey = Self::get_module_chunk(m, compilation);

        // SAFETY: caller will poll the futures
        let s = unsafe { token.used((compilation, m, chunk_ukey, info)) };
        s.spawn(
          async move |(compilation, id, chunk_ukey, info)| -> Result<ModuleInfo> {
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
                let module = parse_file_as_module(
                  &fm,
                  Syntax::default(),
                  EsVersion::EsNext,
                  None,
                  &mut errors,
                )
                .expect("parse failed");
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
        let new_name = find_new_name(&symbol, all_used_names, &vec![]);
        all_used_names.insert(new_name.clone());
        new_name
      } else {
        all_used_names.insert(symbol.clone());
        symbol.clone()
      };

      require_info.required_symbol = Some(new_name.clone());
    }

    require_info
  }

  #[allow(clippy::too_many_arguments)]
  fn link_module_exports(
    &self,
    current: ModuleIdentifier,
    current_chunk: ChunkUkey,
    compilation: &Compilation,
    concate_modules_map: &mut IdentifierIndexMap<ModuleInfo>,
    required: &mut IdentifierIndexMap<ExternalInterop>,
    link: &mut UkeyMap<ChunkUkey, ChunkLinkContext>,
    needed_namespace_objects: &mut IdentifierIndexSet,
    entry_imports: &mut IdentifierIndexMap<FxHashMap<Atom, Atom>>,
    exports: &mut UkeyMap<ChunkUkey, ExportsContext>,
    escaped_identifiers: &FxHashMap<String, Vec<String>>,
    keep_export_name: bool,
    link_re_export: bool,
  ) -> Vec<Diagnostic> {
    let mut errors = vec![];
    let context = &compilation.options.context;
    let module_graph = compilation.get_module_graph();
    let info = concate_modules_map
      .get_mut(&current)
      .unwrap_or_else(|| panic!("should have module info for module: {current}"));

    match info {
      ModuleInfo::Concatenated(info) => {
        if let Some(export_map) = info.export_map.as_ref() {
          for (export_name, local) in export_map {
            let local = local.clone().into();
            let internal_name = info.get_internal_name(&local).unwrap_or_else(|| {
              panic!(
                "{} should have internal name for exported member: {local}, internal_names: {:?}",
                info.module, &info.internal_names
              )
            });

            // the real chunk needs export the symbol
            let exported = Self::add_chunk_export(
              current_chunk,
              internal_name.clone(),
              export_name.clone(),
              exports,
              keep_export_name,
            );

            if exported.is_none() && keep_export_name {
              errors.push(
                rspack_error::error!(
                  "Entry {current} has conflict exports: {export_name} has already been exported"
                )
                .into(),
              );
            }
          }
        }

        // render inline exports
        let exports_info =
          module_graph.get_prefetched_exports_info(&current, PrefetchExportsInfoMode::Default);

        for (name, export_info) in exports_info.exports() {
          let Some(UsedNameItem::Inlined(inlined)) = export_info.get_used_name(None, None) else {
            continue;
          };

          let inlined_value = inlined.render();
          let chunk_link = link.get_mut_unwrap(&current_chunk);
          let new_name = find_new_name(
            name,
            &chunk_link.used_names,
            &escaped_identifiers[&get_cached_readable_identifier(
              &current,
              &module_graph,
              &compilation.module_static_cache_artifact,
              context,
            )],
          );
          chunk_link.used_names.insert(new_name.clone());
          chunk_link
            .init_fragments
            .push(Box::new(NormalInitFragment::new(
              format!("var {new_name} = {inlined_value};\n"),
              InitFragmentStage::StageConstants,
              0,
              InitFragmentKey::unique(),
              None,
            )));
          Self::add_chunk_export(
            current_chunk,
            new_name,
            name.clone(),
            exports,
            keep_export_name,
          );
        }
      }
      ModuleInfo::External(info) => {
        // entry is wrapped
        // should be rendered as
        //
        // ```js
        // var entry_exports = __webpack_require__('entry')
        // var __export_v = entry_exports.v
        // export { __export_v as export_symbol }
        // ```
        let exports_info = module_graph
          .get_exports_info(&current)
          .as_data(&module_graph);
        let chunk_link = link.get_mut_unwrap(&current_chunk);
        let required_interop = Self::add_require(
          current,
          None,
          Some(info.name.clone().expect("should have name")),
          &mut chunk_link.used_names,
          required,
        );

        // ensure we import the chunk
        entry_imports.entry(current).or_default();

        for (name, export_info) in exports_info.exports() {
          if matches!(export_info.provided(), Some(ExportProvided::NotProvided)) {
            continue;
          }

          if export_info.is_reexport()
            && let Some((Some(dep), _)) = export_info.target().iter().next()
          {
            let module_id = module_graph
              .module_identifier_by_dependency_id(dep)
              .expect("should be ESMExportImportedSpecifierDependency");
            let module = module_graph
              .module_by_identifier(module_id)
              .expect("should have module");
            if module.as_external_module().is_some() {
              // ignore re-exported symbol from external module
              // we have special handle for external module re-exports
              continue;
            }
          }

          let local = match export_info.used_name() {
            Some(UsedNameItem::Inlined(inlined)) => inlined.render().into(),
            Some(UsedNameItem::Str(name)) => {
              required_interop.property_access(name, &mut chunk_link.used_names)
            }
            None => required_interop.property_access(name, &mut chunk_link.used_names),
          };
          Self::add_chunk_export(
            current_chunk,
            local,
            name.clone(),
            exports,
            keep_export_name,
          );
        }
      }
    }

    if link_re_export {
      Self::link_re_export(
        current,
        current_chunk,
        compilation,
        concate_modules_map,
        required,
        link,
        &module_graph,
        needed_namespace_objects,
        entry_imports,
        exports,
      );
    }

    errors
  }

  fn re_export_from_external_module(
    module: &ExternalModule,
    current_chunk: ChunkUkey,
    mode: &ExportMode,
    link: &mut UkeyMap<ChunkUkey, ChunkLinkContext>,
  ) {
    match mode {
      // render export * from 'external module'
      ExportMode::DynamicReexport(_) | ExportMode::EmptyStar(_) => {
        let chunk_link = link.get_mut_unwrap(&current_chunk);

        chunk_link
          .raw_star_exports
          .entry(module.get_request().primary().into())
          .or_default()
          .insert(START_EXPORTS.clone());
      }

      ExportMode::Unused(mode) if mode.name == "*" => {
        let chunk_link = link.get_mut_unwrap(&current_chunk);

        chunk_link
          .raw_star_exports
          .entry(module.get_request().primary().into())
          .or_default()
          .insert(START_EXPORTS.clone());
      }

      ExportMode::ReexportUndefined(_)
      | ExportMode::Missing
      | ExportMode::LazyMake
      | ExportMode::Unused(_) => {}

      ExportMode::ReexportDynamicDefault(_) => {
        let chunk_link = link.get_mut_unwrap(&current_chunk);
        chunk_link.add_re_export_from_request(
          module.get_request().primary().into(),
          JS_DEFAULT_KEYWORD.clone(),
          JS_DEFAULT_KEYWORD.clone(),
        );
      }
      ExportMode::ReexportNamedDefault(mode) => {
        let chunk_link = link.get_mut_unwrap(&current_chunk);
        chunk_link.add_re_export_from_request(
          module.get_request().primary().into(),
          JS_DEFAULT_KEYWORD.clone(),
          mode.name.clone(),
        );
      }
      ExportMode::ReexportNamespaceObject(mode) => {
        let chunk_link = link.get_mut_unwrap(&current_chunk);
        chunk_link
          .raw_star_exports
          .entry(module.get_request().primary().into())
          .or_default()
          .insert(mode.name.clone());
      }
      ExportMode::ReexportFakeNamespaceObject(mode) => {
        let chunk_link = link.get_mut_unwrap(&current_chunk);
        chunk_link
          .raw_star_exports
          .entry(module.get_request().primary().into())
          .or_default()
          .insert(mode.name.clone());
      }
      ExportMode::NormalReexport(normal) => {
        let chunk_link = link.get_mut_unwrap(&current_chunk);
        for item in &normal.items {
          chunk_link.add_re_export_from_request(
            module.get_request().primary().into(),
            item.ids.first().unwrap_or(&item.name).clone(),
            item.name.clone(),
          );
        }
      }
    }
  }

  // export * from 'target'
  // export * as n from 'target'
  #[allow(clippy::too_many_arguments)]
  fn link_re_export(
    current: ModuleIdentifier,
    current_chunk: ChunkUkey,
    compilation: &Compilation,
    concate_modules_map: &mut IdentifierIndexMap<ModuleInfo>,
    required: &mut IdentifierIndexMap<ExternalInterop>,
    link: &mut UkeyMap<ChunkUkey, ChunkLinkContext>,
    module_graph: &ModuleGraph,
    needed_namespace_objects: &mut IdentifierIndexSet,
    entry_imports: &mut IdentifierIndexMap<FxHashMap<Atom, Atom>>,
    exports: &mut UkeyMap<ChunkUkey, ExportsContext>,
  ) {
    let current_module = module_graph
      .module_by_identifier(&current)
      .expect("should have module");

    let all_re_exports = current_module
      .get_dependencies()
      .iter()
      .filter_map(|dep| {
        module_graph
          .dependency_by_id(dep)
          .and_then(|dep| dep.downcast_ref::<ESMExportImportedSpecifierDependency>())
          .and_then(|dep| {
            module_graph
              .connection_by_dependency_id(&dep.id)
              .map(|conn| {
                (
                  dep,
                  module_graph
                    .module_by_identifier(conn.module_identifier())
                    .expect("should have module"),
                )
              })
          })
      })
      .map(|(export_imported_dep, module)| {
        (
          export_imported_dep,
          module,
          export_imported_dep.get_mode(
            module_graph,
            None,
            &compilation.module_graph_cache_artifact,
          ),
        )
      });

    let mut module_re_exports =
      IdentifierIndexMap::<Vec<(&ESMExportImportedSpecifierDependency, ExportMode)>>::default();
    for (export_dep, ref_module, re_exports) in all_re_exports {
      module_re_exports
        .entry(ref_module.identifier())
        .or_default()
        .push((export_dep, re_exports));
    }

    for (orig_ref_module, deps_and_modes) in module_re_exports {
      for (export_dep, re_exports) in deps_and_modes {
        // reset ref_module for each dep
        let ref_module = orig_ref_module;

        let ref_box_module = module_graph
          .module_by_identifier(&ref_module)
          .expect("should have mode");

        if let Some(external_module) = ref_box_module.as_external_module()
          && matches!(
            external_module.get_external_type().as_str(),
            "module-import" | "module"
          )
        {
          Self::re_export_from_external_module(external_module, current_chunk, &re_exports, link);
          continue;
        }

        let chunk_link = link.get_mut_unwrap(&current_chunk);
        match re_exports {
          rspack_core::ExportMode::Missing
          | rspack_core::ExportMode::LazyMake
          | rspack_core::ExportMode::ReexportUndefined(_)
          | rspack_core::ExportMode::EmptyStar(_)
          | rspack_core::ExportMode::Unused(_) => {}

          rspack_core::ExportMode::DynamicReexport(_) => {
            // special handling for export * from normal module
            if export_dep.name.is_none() && export_dep.get_ids(module_graph).is_empty() {
              Self::link_re_export(
                ref_module,
                current_chunk,
                compilation,
                concate_modules_map,
                required,
                link,
                module_graph,
                needed_namespace_objects,
                entry_imports,
                exports,
              );
            }
          }

          rspack_core::ExportMode::ReexportDynamicDefault(mode) => {
            let ref_info = &concate_modules_map[&ref_module];
            if let ModuleInfo::External(ref_info) = ref_info {
              let interop_info = Self::add_require(
                ref_module,
                None,
                Some(ref_info.name.clone().expect("should have name")),
                &mut chunk_link.used_names,
                required,
              );
              let info = &mut concate_modules_map[&ref_module];
              info.set_interop_default_access_used(true);
              let default_exported = interop_info.default_exported(&mut chunk_link.used_names);
              // ensure we import this chunk
              entry_imports.entry(ref_module).or_default();

              Self::add_chunk_export(
                current_chunk,
                default_exported.clone(),
                mode.name.clone(),
                exports,
                true,
              );
            }
          }

          rspack_core::ExportMode::ReexportNamedDefault(mode) => {
            // export { default as n } from './foo.cjs'
            let ref_info = &mut concate_modules_map[&ref_module];
            match ref_info {
              ModuleInfo::External(ref_info) => {
                ref_info.interop_default_access_used = true;
                // var foo_default = /*#__PURE__*/ __webpack_require__.n(m3);
                let interop_info = Self::add_require(
                  ref_module,
                  None,
                  Some(ref_info.name.clone().expect("should have name")),
                  &mut chunk_link.used_names,
                  required,
                );

                let default_exported_symbol =
                  interop_info.default_exported(&mut chunk_link.used_names);

                // ensure we import this chunk
                entry_imports.entry(ref_module).or_default();

                Self::add_chunk_export(
                  current_chunk,
                  default_exported_symbol,
                  mode.name.clone(),
                  exports,
                  true,
                );
              }
              ModuleInfo::Concatenated(_) => {
                // TODO: not found any case for now
              }
            }
          }
          rspack_core::ExportMode::ReexportNamespaceObject(mode) => {
            match &concate_modules_map[&ref_module] {
              ModuleInfo::External(ref_info) => {
                let interop_info = Self::add_require(
                  ref_module,
                  None,
                  Some(ref_info.name.clone().expect("should have name")),
                  &mut chunk_link.used_names,
                  required,
                );

                let namespace = interop_info.namespace(&mut chunk_link.used_names);
                entry_imports.entry(ref_module).or_default();
                Self::add_chunk_export(current_chunk, namespace, mode.name.clone(), exports, true);
              }
              ModuleInfo::Concatenated(ref_info) => {
                // should render ref_info's namespace
                needed_namespace_objects.insert(ref_module);

                // the namespace is already set and de-conflicted
                let namespace = ref_info
                  .namespace_object_name
                  .as_ref()
                  .expect("should have namespace object");

                let ref_chunk = Self::get_module_chunk(ref_module, compilation);

                // ref chunk should expose exported symbol
                let exported = Self::add_chunk_export(
                  ref_chunk,
                  namespace.clone(),
                  mode.name.clone(),
                  exports,
                  ref_chunk == current_chunk,
                );

                if ref_chunk != current_chunk {
                  let chunk_link = link.get_mut_unwrap(&current_chunk);
                  chunk_link.add_re_export(
                    ref_chunk,
                    exported.expect("should have exported"),
                    mode.name.clone(),
                  );
                }
              }
            }
          }
          rspack_core::ExportMode::ReexportFakeNamespaceObject(mode) => {
            let ref_info = &concate_modules_map[&ref_module].as_external();

            let required_interop = Self::add_require(
              ref_module,
              None,
              Some(ref_info.name.clone().expect("should have name")),
              &mut chunk_link.used_names,
              required,
            );

            let namespace_name = required_interop.namespace(&mut chunk_link.used_names);
            entry_imports.entry(ref_module).or_default();

            Self::add_chunk_export(
              current_chunk,
              namespace_name,
              mode.name.clone(),
              exports,
              true,
            );
          }
          rspack_core::ExportMode::NormalReexport(mode) => {
            let exports_info = module_graph.get_exports_info(&ref_module);

            for item in mode.items {
              let mut ref_module = orig_ref_module;

              if item.hidden {
                // ignore hidden
                continue;
              }

              if exports[&current_chunk]
                .exported_symbols
                .contains_key(&item.name)
              {
                continue;
              }

              let name = item.ids.first().unwrap_or(&item.name);
              let mut unknown_export_info = false;
              let mut export_info =
                if let Some(export_info) = exports_info.as_data(module_graph).named_exports(name) {
                  export_info
                } else {
                  unknown_export_info = true;
                  // export info not found, this is likely because the export is from unknown
                  item.export_info.as_data(module_graph)
                };

              let resolved_target = get_target(export_info, module_graph);

              let ids = match resolved_target {
                Some(result) => {
                  ref_module = result.module;
                  if let Some(export_name) = &result.export {
                    export_info = module_graph
                      .get_exports_info(&ref_module)
                      .as_data(module_graph)
                      .named_exports(&export_name[0])
                      .unwrap_or(export_info);
                    export_name.clone()
                  } else {
                    item.ids.clone()
                  }
                }
                None => item.ids.clone(),
              };

              if ref_module != orig_ref_module
                && let Some(external_module) = module_graph
                  .module_by_identifier(&ref_module)
                  .expect("should have module")
                  .as_external_module()
                && matches!(
                  external_module.get_external_type().as_str(),
                  "module-import" | "module"
                )
              {
                // handle external module
                Self::re_export_from_external_module(
                  external_module,
                  current_chunk,
                  &rspack_core::ExportMode::NormalReexport(ExportModeNormalReexport {
                    items: vec![NormalReexportItem {
                      name: item.name.clone(),
                      ids: ids.clone(),
                      hidden: false,
                      checked: item.checked,
                      export_info: item.export_info.clone(),
                    }],
                  }),
                  link,
                );
                continue;
              }

              let chunk_link = link.get_mut_unwrap(&current_chunk);
              let used_name = if unknown_export_info {
                UsedNameItem::Str(item.name.clone())
              } else {
                export_info.get_used_name(None, None).unwrap_or_else(|| {
                  // dynamic export
                  UsedNameItem::Str(item.name.clone())
                })
              };

              if let UsedNameItem::Inlined(inlined) = used_name {
                let new_name = find_new_name(&item.name, &chunk_link.used_names, &vec![]);
                chunk_link.used_names.insert(new_name.clone());
                chunk_link
                  .init_fragments
                  .push(Box::new(NormalInitFragment::new(
                    format!("var {} = /* inlined */ {};\n", new_name, inlined.render()),
                    InitFragmentStage::StageConstants,
                    0,
                    InitFragmentKey::unique(),
                    None,
                  )));
                Self::add_chunk_export(current_chunk, new_name, item.name.clone(), exports, true);
                continue;
              };

              let UsedNameItem::Str(used_name) = used_name else {
                unreachable!()
              };

              // if item is from other module
              let ref_info = &concate_modules_map[&ref_module];

              match ref_info {
                ModuleInfo::External(ref_info) => {
                  let chunk_link = link.get_mut_unwrap(&current_chunk);
                  let interop_info = Self::add_require(
                    ref_info.module,
                    None,
                    Some(ref_info.name.clone().expect("should have name")),
                    &mut chunk_link.used_names,
                    required,
                  );

                  let variable_to_export =
                    find_new_name(&item.name, &chunk_link.used_names, &vec![]);

                  chunk_link.used_names.insert(variable_to_export.clone());
                  entry_imports.entry(ref_info.module).or_default();

                  let variable_to_export =
                    interop_info.property_access(&used_name, &mut chunk_link.used_names);
                  Self::add_chunk_export(
                    current_chunk,
                    variable_to_export,
                    item.name.clone(),
                    exports,
                    true,
                  );
                }
                ModuleInfo::Concatenated(ref_info) => {
                  let Some(internal_name) =
                    ref_info.get_internal_name(ids.first().unwrap_or(&item.name))
                  else {
                    continue;
                  };

                  let ref_chunk = Self::get_module_chunk(ref_module, compilation);

                  let exported = Self::add_chunk_export(
                    ref_chunk,
                    internal_name.clone(),
                    item.name.clone(),
                    exports,
                    ref_chunk == current_chunk,
                  );

                  if ref_chunk != current_chunk {
                    let chunk_link = link.get_mut_unwrap(&current_chunk);
                    chunk_link.add_re_export(
                      ref_chunk,
                      exported.expect("should have export"),
                      item.name.clone(),
                    );
                  }
                }
              }
            }
          }
        }
      }
    }
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
      .chunk_by_ukey
      .keys()
      .map(|chunk| (*chunk, Default::default()))
      .collect::<UkeyMap<ChunkUkey, ExportsContext>>();
    let mut imports = compilation
      .chunk_by_ukey
      .keys()
      .map(|chunk| (*chunk, Default::default()))
      .collect::<UkeyMap<ChunkUkey, IdentifierIndexMap<FxHashMap<Atom, Atom>>>>();

    // const symbol = __webpack_require__(module);
    let mut required = UkeyMap::<ChunkUkey, IdentifierIndexMap<ExternalInterop>>::default();

    // link entry direct exports
    if let Some(preserve_modules) = &self.preserve_modules {
      let modules = module_graph.modules();
      let mut modules = modules.keys().collect::<Vec<_>>();
      modules.sort_by(|a, b| {
        let ad = module_graph.get_depth(a);
        let bd = module_graph.get_depth(b);
        ad.cmp(&bd)
      });
      for module_id in modules {
        if compilation.entry_modules().contains(module_id) {
          continue;
        }

        if compilation
          .chunk_graph
          .get_module_chunks(*module_id)
          .is_empty()
        {
          continue;
        }

        let Some(module) = module_graph
          .module_by_identifier(module_id)
          .expect("should have module")
          .as_normal_module()
        else {
          continue;
        };
        let Some(resource_resolved_data) = module.resource_resolved_data().path() else {
          continue;
        };
        if !resource_resolved_data.starts_with(preserve_modules) {
          continue;
        }

        let chunk = Self::get_module_chunk(*module_id, compilation);
        let required = required.entry(chunk).or_default();
        let needed_namespace = needed_namespace_objects_by_ukey.entry(chunk).or_default();
        let entry_imports = imports
          .get_mut(&chunk)
          .unwrap_or_else(|| panic!("should set imports for chunk {chunk:?}"));

        errors.extend(self.link_module_exports(
          *module_id,
          chunk,
          compilation,
          concate_modules_map,
          required,
          link,
          needed_namespace,
          entry_imports,
          &mut exports,
          escaped_identifiers,
          true,
          true,
        ));
      }
    }

    for (entry_name, entrypoint_ukey) in compilation.entrypoints.iter() {
      let entrypoint = compilation.chunk_group_by_ukey.expect_get(entrypoint_ukey);
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
        .copied()
      {
        let entry_module_chunk = Self::get_module_chunk(entry_module, compilation);
        entry_imports.entry(entry_module).or_default();

        /*
        entry module sometimes are splitted to whatever chunk user needs,
        so the entry chunk maynot actually contains entry modules
         */
        let needs_reexport = entry_module_chunk != entry_chunk_ukey;
        let required = required.entry(entry_module_chunk).or_default();

        errors.extend(self.link_module_exports(
          entry_module,
          entry_module_chunk,
          compilation,
          concate_modules_map,
          required,
          link,
          needed_namespace,
          entry_imports,
          &mut exports,
          escaped_identifiers,
          !needs_reexport,
          false,
        ));

        if needs_reexport {
          // the entry has been moved into other chunk, need to re-export the exports from other chunk
          let entry_info = concate_modules_map
            .get_mut(&entry_module)
            .unwrap_or_else(|| panic!("should have module info for entry module: {entry_module}"));

          match entry_info {
            ModuleInfo::Concatenated(info) => {
              if let Some(export_map) = info.export_map.as_ref() {
                for (export_name, local) in export_map {
                  let local = local.clone().into();
                  let internal_name = info.get_internal_name(&local).unwrap_or_else(|| {
                  panic!("{} should have internal name for exported member: {local}, internal_names: {:?}", info.module, &info.internal_names)
                });

                  // the real chunk needs export the symbol
                  let exported = Self::add_chunk_export(
                    entry_module_chunk,
                    internal_name.clone(),
                    export_name.clone(),
                    &mut exports,
                    false,
                  )
                  .expect("no strict export should always success");

                  let entry_chunk_link = link.get_mut_unwrap(&entry_chunk_ukey);
                  entry_chunk_link.add_re_export(
                    entry_module_chunk,
                    exported.clone(),
                    export_name.clone(),
                  );
                }
              }

              // render inline exports
              let exports_info = module_graph
                .get_prefetched_exports_info(&entry_module, PrefetchExportsInfoMode::Default);

              for (name, export_info) in exports_info.exports() {
                let Some(UsedNameItem::Inlined(_)) = export_info.get_used_name(None, None) else {
                  continue;
                };

                // find the local symbol of the exported chunk
                let exported =
                  Self::chunk_symbol_for_export(entry_module_chunk, name, &mut exports);

                if let Some(exported) = exported {
                  let entry_chunk_link = link.get_mut_unwrap(&entry_chunk_ukey);
                  entry_chunk_link.add_re_export(
                    entry_module_chunk,
                    exported.clone(),
                    name.clone(),
                  );
                }
              }
            }
            ModuleInfo::External(_) => {
              // entry is wrapped
              // should be rendered as
              //
              // ```js
              // var entry_exports = __webpack_require__('entry')
              // var __export_v = entry_exports.v
              // export { __export_v as export_symbol }
              // ```
              let exports_info = module_graph
                .get_exports_info(&entry_module)
                .as_data(&module_graph);
              // ensure we import the chunk
              entry_imports.entry(entry_module).or_default();

              for (name, export_info) in exports_info.exports() {
                if matches!(export_info.provided(), Some(ExportProvided::NotProvided)) {
                  continue;
                }

                // find the local symbol of the exported chunk
                let exported =
                  Self::chunk_symbol_for_export(entry_module_chunk, name, &mut exports);

                if let Some(exported) = exported {
                  let entry_chunk_link = link.get_mut_unwrap(&entry_chunk_ukey);
                  entry_chunk_link.add_re_export(
                    entry_module_chunk,
                    exported.clone(),
                    name.clone(),
                  );
                }
              }
            }
          }
        }

        Self::link_re_export(
          entry_module,
          entry_chunk_ukey,
          compilation,
          concate_modules_map,
          required,
          link,
          &module_graph,
          needed_namespace,
          entry_imports,
          &mut exports,
        );
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
          let Some(dep) = module_graph.dependency_by_id(dep_id) else {
            continue;
          };

          let Some(conn) = module_graph.connection_by_dependency_id(dep_id) else {
            continue;
          };
          if !conn.is_target_active(
            &module_graph,
            None,
            &compilation.module_graph_cache_artifact,
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
              &module_graph,
              &compilation.module_graph_cache_artifact,
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
              &module_graph,
              &compilation.module_graph_cache_artifact,
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
              let ref_chunk = Self::get_module_chunk(module_id, compilation);
              let ref_external = concate_modules_map[ref_module].is_external();

              if from_other_chunk && !ref_external {
                let exported = Self::add_chunk_export(
                  ref_chunk,
                  symbol_binding.symbol.clone(),
                  symbol_binding.symbol.clone(),
                  &mut exports,
                  false,
                );

                symbol_binding.symbol = exported.expect("should have exported");
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
            inline_refs.insert(ref_string.clone(), Ref::Inline(inlined_string));
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
            &module_graph,
            &compilation.module_static_cache_artifact,
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
            &module_graph,
            None,
            &compilation.module_graph_cache_artifact,
          ) {
            continue;
          }

          let ref_module = conn.module_identifier();
          chunk_imports.entry(*ref_module).or_default();
        }
      }
    }

    /*
    This is a hack for external module.
    export * from 'external module'
    will generate raw_import for external module request
    and also generate raw_star_export for it.

    we can remove the empty raw_import if there is star reexport.
    */
    for chunk_link in link.values_mut() {
      for (source, _) in &chunk_link.raw_star_exports {
        let key = (source.clone(), None);
        if let Some(import_spec) = chunk_link.raw_import_stmts.get(&key)
          && import_spec.atoms.is_empty()
          && import_spec.default_import.is_none()
        {
          chunk_link.raw_import_stmts.swap_remove(&key);
        }
      }
    }

    // put result into chunk_link context
    for (chunk, exports) in exports {
      *link
        .get_mut(&chunk)
        .expect("should have chunk")
        .exports_mut() = exports.exports;
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
    info_id: &ModuleIdentifier,
    mut export_name: Vec<Atom>,
    module_to_info_map: &mut IdentifierIndexMap<ModuleInfo>,
    needed_namespace_objects: &mut IdentifierIndexSet,
    as_call: bool,
    call_context: bool,
    strict_esm_module: bool,
    asi_safe: Option<bool>,
    already_visited: &mut FxHashSet<MaybeDynamicTargetExportInfoHashKey>,
    required: &mut IdentifierIndexMap<ExternalInterop>,
    all_used_names: &mut FxHashSet<Atom>,
  ) -> Ref {
    let module = mg
      .module_by_identifier(info_id)
      .expect("should have module");
    let exports_type = module.get_exports_type(mg, mg_cache, strict_esm_module);
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

    let exports_info =
      mg.get_prefetched_exports_info(info_id, PrefetchExportsInfoMode::Nested(&export_name));

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
              None,
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
                return Ref::Inline(format!(
                  "{} {}",
                  to_normal_comment(&format!(
                    "inlined export {}",
                    property_access(&export_name, 0)
                  )),
                  inlined.render()
                ));
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

        let reexport = export_info.find_target(
          mg,
          Arc::new(|module: &ModuleIdentifier| module_to_info_map.contains_key(module)),
        );
        match reexport {
          FindTargetResult::NoTarget => {}
          FindTargetResult::InvalidTarget(target) => {
            if let Some(export) = target.export {
              let exports_info = mg.get_prefetched_exports_info(
                &target.module,
                PrefetchExportsInfoMode::Nested(&export),
              );
              if let Some(UsedName::Inlined(inlined)) = ExportsInfoGetter::get_used_name(
                GetUsedNameParam::WithNames(&exports_info),
                None,
                &export,
              ) {
                return Ref::Inline(format!(
                  "{} {}",
                  to_normal_comment(&format!(
                    "inlined export {}",
                    property_access(&export_name, 0)
                  )),
                  inlined.inlined_value().render()
                ));
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
                .module_by_identifier(&ref_info.id())
                .expect("should have module")
                .build_meta();

              return Self::get_binding(
                from,
                mg,
                mg_cache,
                &ref_info.id(),
                if let Some(reexport_export) = reexport.export {
                  [reexport_export.clone(), export_name[1..].to_vec()].concat()
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
            UsedName::Inlined(inlined) => Ref::Inline(inlined.render()),
          };
        }

        if let Some(UsedName::Inlined(inlined)) = used_name {
          let comment = to_normal_comment(&format!(
            "inlined export {}",
            property_access(&export_name, 0)
          ));
          return Ref::Inline(format!("{comment}{}", inlined.render()));
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
                None,
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
            UsedName::Inlined(inlined) => {
              let comment = to_normal_comment(&format!(
                "inlined export {}",
                property_access(&export_name, 0)
              ));
              Ref::Inline(format!("{}{comment}", inlined.render()))
            }
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
