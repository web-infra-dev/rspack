use std::{collections::hash_map::Entry, sync::Arc};

use rspack_collections::{IdentifierIndexMap, IdentifierIndexSet, IdentifierMap, UkeyMap};
use rspack_core::{
  BuildMetaDefaultObject, BuildMetaExportsType, ChunkInitFragments, ChunkLinkContext, ChunkUkey,
  Compilation, ConcatenatedModule, ConcatenatedModuleIdent, ExportInfoGetter, ExportProvided,
  ExportsInfoGetter, ExportsType, FindTargetResult, GetUsedNameParam, IdentCollector,
  MaybeDynamicTargetExportInfoHashKey, ModuleGraph, ModuleGraphCacheArtifact, ModuleIdentifier,
  ModuleInfo, NAMESPACE_OBJECT_EXPORT, PathData, PrefetchExportsInfoMode, Ref, RuntimeGlobals,
  SourceType, SymbolRef, UsageState, UsedName, UsedNameItem, find_new_name,
  get_js_chunk_filename_template, property_access, property_name, reserved_names::RESERVED_NAMES,
  returning_function, rspack_sources::ReplaceSource, to_normal_comment,
};
use rspack_error::Result;
use rspack_javascript_compiler::ast::Ast;
use rspack_plugin_javascript::{JsPlugin, RenderSource, visitors::swc_visitor::resolver};
use rspack_util::{
  atom::Atom,
  fx_hash::{FxHashMap, FxHashSet, FxIndexMap},
  swc::join_atom,
};
use swc_core::{
  common::{FileName, SyntaxContext},
  ecma::{
    ast::{EsVersion, Program},
    parser::{Syntax, parse_file_as_module},
  },
};

use crate::EsmLibraryPlugin;

impl EsmLibraryPlugin {
  pub(crate) async fn link(&self, compilation: &mut Compilation) -> Result<()> {
    let module_graph = compilation.get_module_graph();

    // codegen uses self.concatenated_modules_map_for_codegen which has hold another Arc, so
    // it's safe to access concate_modules_map lock
    let mut concate_modules_map = self.concatenated_modules_map.lock().await;
    let concate_modules_map = Arc::get_mut(
      concate_modules_map
        .get_mut(&compilation.id().0)
        .expect("should has compilation"),
    )
    .expect("should have unique access to concatenated modules map");

    concate_modules_map.retain(|m, _| {
      // remove orphan modules
      !Self::is_orphan(*m, compilation)
    });

    // analyze every modules and collect identifiers to concate_modules_map
    self
      .analyze_module(compilation, concate_modules_map)
      .await?;

    // initialize data for link chunks
    let mut link: UkeyMap<ChunkUkey, ChunkLinkContext> = compilation
      .chunk_by_ukey
      .keys()
      .map(|ukey| {
        let modules = compilation.chunk_graph.get_chunk_modules_identifier(ukey);

        let mut hoisted_modules = modules
          .iter()
          .copied()
          .filter(|m| {
            let info = concate_modules_map
              .get(m)
              .expect("should have set module info");
            matches!(info, ModuleInfo::Concatenated(_))
          })
          .collect::<Vec<_>>();

        // sort modules based on the post order index
        hoisted_modules.sort_by(|m1, m2| {
          let m1_index = module_graph.get_post_order_index(m1);
          let m2_index = module_graph.get_post_order_index(m2);
          m1_index.cmp(&m2_index)
        });
        let chunk_link = ChunkLinkContext {
          hoisted_modules: hoisted_modules.into_iter().collect(),
          ..Default::default()
        };

        (*ukey, chunk_link)
      })
      .collect();

    for (chunk_ukey, chunk_link) in &mut link {
      self.deconflict_symbols(compilation, *chunk_ukey, concate_modules_map, chunk_link);
    }

    // link imported specifier with exported symbol
    let mut needed_namespace_objects = IdentifierIndexSet::default();
    self.link_imports_and_exports(
      compilation,
      &mut link,
      concate_modules_map,
      &mut needed_namespace_objects,
    );

    let mut namespace_object_sources: IdentifierMap<String> = IdentifierMap::default();
    let mut visited = FxHashSet::default();

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

        let module_info = concate_modules_map
          .get(module_info_id)
          .map(|m| m.as_concatenated())
          .expect("should have module info");

        let module_graph = compilation.get_module_graph();
        let box_module = module_graph
          .module_by_identifier(module_info_id)
          .expect("should have box module");
        let module_readable_identifier =
          box_module.readable_identifier(&compilation.options.context);
        let strict_esm_module = box_module.build_meta().strict_esm_module;
        let name_space_name = module_info.namespace_object_name.clone();

        if let Some(ref _namespace_export_symbol) = module_info.namespace_export_symbol {
          continue;
        }

        let mut ns_obj = Vec::new();
        let exports_info = module_graph.get_exports_info(module_info_id);
        for export_info in exports_info.as_data(&module_graph).exports() {
          if matches!(
            export_info.as_data(&module_graph).provided(),
            Some(ExportProvided::NotProvided)
          ) {
            continue;
          }

          if let Some(UsedNameItem::Str(used_name)) =
            ExportInfoGetter::get_used_name(export_info.as_data(&module_graph), None, None)
          {
            let final_name = ConcatenatedModule::get_final_name(
              &compilation.get_module_graph(),
              &compilation.module_graph_cache_artifact,
              module_info_id,
              vec![
                export_info
                  .as_data(&module_graph)
                  .name()
                  .cloned()
                  .unwrap_or("".into()),
              ],
              concate_modules_map,
              None,
              &mut needed_namespace_objects,
              false,
              false,
              strict_esm_module,
              Some(true),
              &compilation.options.context,
            );

            ns_obj.push(format!(
              "\n  {}: {}",
              property_name(&used_name).expect("should have property_name"),
              returning_function(&compilation.options.output.environment, &final_name, "")
            ));
          }
        }
        // https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/optimize/ConcatenatedModule.js#L1539
        let name = name_space_name.expect("should have name_space_name");
        let define_getters = if !ns_obj.is_empty() {
          format!(
            "{}({}, {{ {} }});\n",
            RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
            name,
            ns_obj.join(",")
          )
        } else {
          String::new()
        };

        let module_info = concate_modules_map
          .get_mut(module_info_id)
          .map(|m| m.as_concatenated_mut())
          .expect("should have module info");

        if !ns_obj.is_empty() {
          module_info
            .runtime_requirements
            .insert(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);
        }

        namespace_object_sources.insert(
          *module_info_id,
          format!(
            "// NAMESPACE OBJECT: {}\nvar {} = {{}};\n{}({});\n{}\n",
            module_readable_identifier,
            name,
            RuntimeGlobals::MAKE_NAMESPACE_OBJECT,
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

    for (module, namespace_source) in namespace_object_sources {
      let info = concate_modules_map
        .get_mut(&module)
        .expect("should have module info")
        .as_concatenated_mut();
      info.namespace_object_source = Some(namespace_source);
    }

    compilation.chunk_graph.link = Some(link);
    Ok(())
  }

  pub fn is_orphan(m: ModuleIdentifier, compilation: &Compilation) -> bool {
    compilation.chunk_graph.get_module_chunks(m).is_empty()
  }

  pub fn get_module_chunk(m: ModuleIdentifier, compilation: &Compilation) -> ChunkUkey {
    let chunks = compilation.chunk_graph.get_module_chunks(m);
    if chunks.is_empty() {
      panic!("module {} is not in any chunk", m);
    }

    if chunks.len() > 1 {
      unimplemented!("module is in multiple chunks");
    }

    *chunks.iter().next().expect("at least one chunk")
  }

  fn deconflict_symbols(
    &self,
    compilation: &Compilation,
    chunk_ukey: ChunkUkey,
    concate_modules_map: &mut IdentifierIndexMap<ModuleInfo>,
    chunk_link: &mut ChunkLinkContext,
  ) {
    let module_graph: rspack_core::ModuleGraph<'_> = compilation.get_module_graph();
    let mut all_used_names: FxHashSet<Atom> = RESERVED_NAMES
      .iter()
      .map(|s| Atom::new(*s))
      .chain(chunk_link.hoisted_modules.iter().flat_map(|m| {
        let info = concate_modules_map.get(m).expect("should have info");
        info
          .as_concatenated()
          .global_scope_ident
          .iter()
          .map(|ident| ident.id.sym.clone())
      }))
      .collect();

    // merge all all_used_names from hoisted modules
    for id in &chunk_link.hoisted_modules {
      let concate_info = concate_modules_map
        .get_mut(id)
        .expect("should have info")
        .as_concatenated();
      all_used_names.extend(concate_info.all_used_names.clone());
    }

    // deconflict top level symbols
    for id in &chunk_link.hoisted_modules {
      let module = module_graph
        .module_by_identifier(id)
        .expect("should have module");
      let exports_type = module.build_meta().exports_type;
      let default_object = module.build_meta().default_object;
      let readable_identifier = module.readable_identifier(&compilation.options.context);
      let info = concate_modules_map
        .get_mut(id)
        .expect("should have module info");

      // Handle additional logic based on module build meta
      if exports_type != BuildMetaExportsType::Namespace {
        let external_name_interop: Atom = find_new_name(
          "namespaceObject",
          &all_used_names,
          None,
          &readable_identifier,
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
          None,
          &readable_identifier,
        );
        all_used_names.insert(external_name_interop.clone());
        info.set_interop_namespace_object2_name(Some(external_name_interop.clone()));
      }

      if matches!(
        exports_type,
        BuildMetaExportsType::Dynamic | BuildMetaExportsType::Unset
      ) {
        let external_name_interop: Atom =
          find_new_name("default", &all_used_names, None, &readable_identifier);
        all_used_names.insert(external_name_interop.clone());
        info.set_interop_default_access_name(Some(external_name_interop.clone()));
      }

      let mut internal_names = FxHashMap::default();
      let concate_info = info.as_concatenated();

      for (atom, ctxt) in concate_info.binding_to_ref.keys() {
        // only need to handle top level scope
        if ctxt != &concate_info.module_ctxt {
          continue;
        }

        if all_used_names.contains(atom) {
          let new_name = find_new_name(atom, &all_used_names, None, &readable_identifier);
          all_used_names.insert(new_name.clone());
          internal_names.insert(atom.clone(), new_name);
        } else {
          all_used_names.insert(atom.clone());
          internal_names.insert(atom.clone(), atom.clone());
        }
      }

      let concate_info = info.as_concatenated_mut();
      concate_info.internal_names = internal_names;

      // Handle the name passed through by namespace_export_symbol
      if let Some(ref namespace_export_symbol) = concate_info.namespace_export_symbol
        && namespace_export_symbol.starts_with(NAMESPACE_OBJECT_EXPORT)
        && namespace_export_symbol.len() > NAMESPACE_OBJECT_EXPORT.len()
      {
        let name = Atom::from(namespace_export_symbol[NAMESPACE_OBJECT_EXPORT.len()..].to_string());
        all_used_names.insert(name.clone());
        concate_info
          .internal_names
          .insert(namespace_export_symbol.clone(), name.clone());
      }

      // Handle namespaceObjectName for concatenated type
      let namespace_object_name =
        if let Some(ref namespace_export_symbol) = concate_info.namespace_export_symbol {
          concate_info
            .internal_names
            .get(namespace_export_symbol)
            .cloned()
        } else {
          Some(find_new_name(
            "namespaceObject",
            &all_used_names,
            None,
            &readable_identifier,
          ))
        };
      if let Some(namespace_object_name) = namespace_object_name {
        all_used_names.insert(namespace_object_name.clone());
        concate_info.namespace_object_name = Some(namespace_object_name.clone());
      }
    }

    for external_module in compilation
      .chunk_graph
      .get_chunk_modules(&chunk_ukey, &module_graph)
      .into_iter()
      .filter(|m| !chunk_link.hoisted_modules.contains(&m.identifier()))
    {
      let ModuleInfo::External(info) = concate_modules_map
        .get_mut(&external_module.identifier())
        .expect("should have external module info")
      else {
        unreachable!("should be un-scope-hoisted module");
      };
      let name = find_new_name(
        "",
        &all_used_names,
        None,
        &external_module.readable_identifier(&compilation.options.context),
      );
      info.name = Some(name.clone());
      all_used_names.insert(name.clone());
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
            .chunk_id_optional(
              chunk
                .id(&compilation.chunk_ids_artifact)
                .map(|id| id.as_str()),
            )
            .chunk_name_optional(chunk.name_for_filename_template(&compilation.chunk_ids_artifact))
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
        let chunk_ukey = Self::get_module_chunk(m, compilation);

        // SAFETY: caller will poll the futures
        let s = unsafe { token.used((compilation, m, chunk_ukey, info)) };
        s.spawn(
          async move |(compilation, id, chunk_ukey, info)| -> Result<ModuleInfo> {
            let module_graph = compilation.get_module_graph();

            match info {
              rspack_core::ModuleInfo::External(mut external_module_info) => {
                // we use Object.assign(__webpack_require__.m, {...}) to register modules
                external_module_info
                  .runtime_requirements
                  .insert(RuntimeGlobals::REQUIRE | RuntimeGlobals::MODULE_FACTORIES);
                Ok(ModuleInfo::External(external_module_info))
              }
              rspack_core::ModuleInfo::Concatenated(mut concate_info) => {
                let hooks = JsPlugin::get_compilation_hooks(compilation.id());

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
                      .expect("should have module"),
                    &mut render_source,
                    &mut chunk_init_fragments,
                  )
                  .await?;
                concate_info = Box::new(
                  codegen_res
                    .concatenation_scope
                    .as_ref()
                    .expect("should have concatenation scope")
                    .current_module
                    .clone(),
                );

                let m = module_graph
                  .module_by_identifier(&id)
                  .expect("should have module");
                let cm: Arc<swc_core::common::SourceMap> = Default::default();
                let readable_identifier = m.readable_identifier(&compilation.options.context);
                let fm = cm.new_source_file(
                  Arc::new(FileName::Custom(readable_identifier.clone().into_owned())),
                  render_source.source.source().to_string(),
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

                let mut binding_to_ref: FxHashMap<
                  (Atom, SyntaxContext),
                  Vec<ConcatenatedModuleIdent>,
                > = FxHashMap::default();

                for ident in &idents {
                  match binding_to_ref.entry((ident.id.sym.clone(), ident.id.ctxt)) {
                    Entry::Occupied(mut occ) => {
                      occ.get_mut().push(ident.clone());
                    }
                    Entry::Vacant(vac) => {
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
                concate_info.star_exports = codegen_res
                  .concatenation_scope
                  .as_ref()
                  .expect("should have concatenation scope")
                  .star_exports
                  .clone();
                concate_info.ast = Some(ast);
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
                  .extend(chunk_init_fragments);
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

    let module_graph = compilation.get_module_graph();
    let concate_modules_map = orig_concate_modules_map;

    concate_modules_map.iter_mut().for_each(|(id, info)| {
      let module = module_graph
        .module_by_identifier(id)
        .expect("should have module");
      let readable_identifier = module.readable_identifier(&compilation.options.context);
      let exports_type = module.build_meta().exports_type;
      let default_object = module.build_meta().default_object;
      // Handle additional logic based on module build meta
      if exports_type != BuildMetaExportsType::Namespace {
        let external_name_interop: Atom = "namespaceObject".into();
        info.set_interop_namespace_object_name(Some(external_name_interop.clone()));
      }

      if exports_type == BuildMetaExportsType::Default
        && !matches!(default_object, BuildMetaDefaultObject::Redirect)
      {
        let external_name_interop: Atom = "namespaceObject2".into();
        info.set_interop_namespace_object2_name(Some(external_name_interop.clone()));
      }

      if matches!(
        exports_type,
        BuildMetaExportsType::Dynamic | BuildMetaExportsType::Unset
      ) {
        let external_name_interop: Atom = format!("{readable_identifier}_default").into();
        info.set_interop_default_access_name(Some(external_name_interop.clone()));
      }
    });

    Ok(())
  }

  fn link_imports_and_exports(
    &self,
    compilation: &Compilation,
    link: &mut UkeyMap<ChunkUkey, ChunkLinkContext>,
    concate_modules_map: &mut IdentifierIndexMap<ModuleInfo>,
    needed_namespace_objects: &mut IdentifierIndexSet,
  ) {
    let module_graph: rspack_core::ModuleGraph<'_> = compilation.get_module_graph();
    let mut exports = UkeyMap::<ChunkUkey, IdentifierMap<FxHashSet<Atom>>>::default();
    let mut imports = UkeyMap::<ChunkUkey, IdentifierIndexMap<FxHashMap<Atom, Atom>>>::default();

    // const symbol = __webpack_require__(module);
    let mut required = UkeyMap::<ChunkUkey, IdentifierMap<Atom>>::default();

    // link entry direct exports
    for entrypoint_ukey in compilation.entrypoints.values() {
      let entrypoint = compilation.chunk_group_by_ukey.expect_get(&entrypoint_ukey);
      let entry_chunk_ukey = entrypoint.get_entrypoint_chunk();
      let chunk_link = link
        .get_mut(&entry_chunk_ukey)
        .expect("should have link for chunk");

      for entry_module in compilation
        .chunk_graph
        .get_chunk_entry_modules(&entry_chunk_ukey)
      {
        let info = concate_modules_map
          .get(&entry_module)
          .expect("should have module info");
        let Some(info) = info.try_as_concatenated() else {
          continue;
        };

        if let Some(export_map) = info.export_map.as_ref() {
          for export_atom in export_map.keys() {
            chunk_link
              .exports
              .entry(entry_module)
              .or_default()
              .insert(export_atom.clone());
          }
        }

        for (exported_module, mode) in &info.star_exports {
          let needs_require = matches!(
            concate_modules_map
              .get(exported_module)
              .expect("should have info"),
            ModuleInfo::External(_)
          );

          match mode {
            rspack_core::ExportMode::EmptyStar(_)
            | rspack_core::ExportMode::Unused(_)
            | rspack_core::ExportMode::Missing => {}

            // export { default as name } from './dynamic'
            rspack_core::ExportMode::ReexportDynamicDefault(mode) => {
              dbg!(exported_module, &mode.name);
              todo!()
            }

            rspack_core::ExportMode::ReexportNamedDefault(export_mode_reexport_named_default) => {
              todo!()
            }
            rspack_core::ExportMode::ReexportNamespaceObject(
              export_mode_reexport_namespace_object,
            ) => todo!(),
            rspack_core::ExportMode::ReexportFakeNamespaceObject(
              export_mode_fake_namespace_object,
            ) => todo!(),
            rspack_core::ExportMode::ReexportUndefined(export_mode_reexport_undefined) => todo!(),
            rspack_core::ExportMode::NormalReexport(export_mode_normal_reexport) => todo!(),
            rspack_core::ExportMode::DynamicReexport(export_mode_dynamic_reexport) => todo!(),
          }

          let exports_info = module_graph
            .get_exports_info(exported_module)
            .as_data(&module_graph);

          for export in exports_info.exports() {
            let export = export.as_data(&module_graph);
            if matches!(export.global_used(), Some(UsageState::Unused)) {
              continue;
            }
            dbg!(exported_module, export.name());
            if let Some(name) = export.name() {
              chunk_link
                .exports
                .entry(*exported_module)
                .or_default()
                .insert(name.clone());
            }
          }
        }
      }
    }

    // calculate exports based on imports
    for (chunk, chunk_link) in link.iter_mut() {
      let mut refs = FxHashMap::default();

      for m in chunk_link.hoisted_modules.clone() {
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

            let module = module_graph
              .module_by_identifier(ref_module)
              .expect("should have ref module");

            let mut binding = Self::get_binding(
              &module_graph,
              &compilation.module_graph_cache_artifact,
              ref_module,
              options.ids.clone(),
              concate_modules_map,
              needed_namespace_objects,
              options.call,
              !options.direct_import,
              module.build_info().strict,
              options.asi_safe,
              &mut Default::default(),
            );

            if let Ref::Symbol(symbol_binding) = &mut binding {
              let module_id = symbol_binding.module;
              let ref_chunk = Self::get_module_chunk(module_id, compilation);

              if &ref_chunk != chunk {
                // ref chunk should expose the symbol
                exports
                  .entry(ref_chunk)
                  .or_default()
                  .entry(module_id)
                  .or_default()
                  .insert(symbol_binding.symbol.clone());
              }
            }

            refs.insert(
              ref_string
                .strip_suffix("._")
                .expect("should have prefix: '._'")
                .to_string(),
              binding,
            );
          }
        }

        for (ref_module, refs) in &concatenation_scope.dyn_refs {
          let ref_chunk = Self::get_module_chunk(*ref_module, compilation);
          if &ref_chunk != chunk {
            exports
              .entry(ref_chunk)
              .or_default()
              .entry(*ref_module)
              .or_default()
              .extend(refs.clone());
          }
        }
      }

      // deconflict imported symbol and required symbols
      let mut ref_by_symbol =
        FxIndexMap::<(Atom, ModuleIdentifier), Vec<(String, SymbolRef)>>::default();
      refs
        .into_iter()
        .filter_map(|(ref_string, binding_ref)| {
          if let Ref::Symbol(symbol) = binding_ref {
            Some((ref_string, symbol))
          } else {
            None
          }
        })
        .for_each(|(ref_string, symbol_ref)| {
          ref_by_symbol
            .entry((symbol_ref.symbol.clone(), symbol_ref.module))
            .or_default()
            .push((ref_string, symbol_ref));
        });

      let mut refs = FxHashMap::default();
      let all_used_names = &mut chunk_link.used_names;

      for ((symbol, m), mut all_refs) in ref_by_symbol {
        let ref_chunk = Self::get_module_chunk(m, compilation);
        let needs_require = matches!(concate_modules_map.get(&m), Some(ModuleInfo::External(_)));
        let needs_import_chunk = ref_chunk != *chunk;

        // check if we should import this symbol from other chunk
        if needs_import_chunk || needs_require {
          let (orig_symbol, local_symbol) = if all_used_names.contains(&symbol) {
            let new_symbol = find_new_name(&symbol, &all_used_names, None, "");
            all_used_names.insert(new_symbol.clone());

            for (_, cur_ref) in &mut all_refs {
              cur_ref.symbol = new_symbol.clone();
            }

            (symbol.clone(), new_symbol)
          } else {
            all_used_names.insert(symbol.clone());
            (symbol.clone(), symbol.clone())
          };

          if needs_require {
            imports.entry(*chunk).or_default(); // make sure we imported this chunk, so that runtime can register this module
            required.entry(*chunk).or_default().insert(m, local_symbol);
          } else {
            // import symbol from that chunk
            imports
              .entry(*chunk)
              .or_default()
              .entry(m)
              .or_default()
              .insert(orig_symbol, local_symbol);
          }
        }

        for (ref_str, cur_ref) in all_refs {
          refs.insert(ref_str, Ref::Symbol(cur_ref));
        }
      }

      chunk_link.needed_namespace_objects = needed_namespace_objects.clone();
      chunk_link.refs = refs;
    }

    // put result into chunk_link context
    for (chunk, exports) in exports {
      link.entry(chunk).or_default().exports = exports;
    }
    for (chunk, imports) in imports {
      link.entry(chunk).or_default().imports = imports;
    }
    for (chunk, required) in required {
      link.entry(chunk).or_default().required = required;
    }
  }

  // if imported specifier is in the same chunk
  // the final name is symbol in current chunk
  //
  // if imported specifier is in other chunk
  // the final name is symbol in referenced chunk
  fn get_binding(
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
  ) -> Ref {
    let info = module_to_info_map
      .get(info_id)
      .expect("should have module info");

    let module = mg
      .module_by_identifier(&info.id())
      .expect("should have module");
    let exports_type = module.get_exports_type(mg, mg_cache, strict_esm_module);

    if export_name.is_empty() {
      match exports_type {
        ExportsType::DefaultOnly => {
          // shadowing the previous immutable ref to avoid violating rustc borrow rules
          let info = module_to_info_map
            .get_mut(info_id)
            .expect("should have module info");
          info.set_interop_namespace_object2_used(true);

          return Ref::Symbol(SymbolRef::new(
            info.id(),
            info
              .get_interop_namespace_object2_name()
              .cloned()
              .expect("should have raw name"),
            export_name.clone(),
            Arc::new(move |binding| binding.symbol.to_string()),
          ));
        }
        ExportsType::DefaultWithNamed => {
          // shadowing the previous immutable ref to avoid violating rustc borrow rules
          let info = module_to_info_map
            .get_mut(info_id)
            .expect("should have module info");
          info.set_interop_namespace_object_used(true);

          return Ref::Symbol(SymbolRef::new(
            info.id(),
            info
              .get_interop_namespace_object_name()
              .expect("should have interop_namespace_object_name")
              .clone(),
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
            let info = module_to_info_map
              .get_mut(info_id)
              .expect("should have module info");
            info.set_interop_default_access_used(true);
            export_name = export_name[1..].to_vec();

            let default_access_name = info
              .get_interop_default_access_name()
              .cloned()
              .expect("should have default access name");

            return Ref::Symbol(SymbolRef::new(
              info.id(),
              default_access_name,
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

    if export_name.is_empty() {
      match info {
        ModuleInfo::Concatenated(info) => {
          needed_namespace_objects.insert(info.module);
          return Ref::Symbol(SymbolRef::new(
            info.module,
            info
              .namespace_object_name
              .clone()
              .expect("should have namespace_object_name"),
            export_name.clone(),
            Arc::new(move |binding| normal_render(binding, as_call, call_context, asi_safe)),
          ));
        }
        ModuleInfo::External(info) => {
          return Ref::Symbol(SymbolRef::new(
            info.module,
            info.name.clone().expect("should have symbol"),
            export_name.clone(),
            Arc::new(move |binding| normal_render(binding, as_call, call_context, asi_safe)),
          ));
        }
      }
    }

    let exports_info = mg.get_prefetched_exports_info(
      &info.id(),
      PrefetchExportsInfoMode::NamedNestedExports(&export_name),
    );

    let export_info = exports_info.get_export_info_without_mut_module_graph(&export_name[0]);
    let export_info_hash_key = export_info.as_hash_key();

    if already_visited.contains(&export_info_hash_key) {
      return Ref::Inline("/* circular reexport */ Object(function x() { x() }())".into());
    }

    already_visited.insert(export_info_hash_key);

    match info {
      ModuleInfo::Concatenated(info) => {
        let export_id = export_name.first().cloned();
        if matches!(export_info.provided(), Some(ExportProvided::NotProvided)) {
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

        if let Some(ref export_id) = export_id
          && let Some(direct_export) = info.export_map.as_ref().and_then(|map| map.get(export_id))
        {
          if let Some(used_name) = ExportsInfoGetter::get_used_name(
            GetUsedNameParam::WithNames(&exports_info),
            None,
            &export_name,
          ) {
            match used_name {
              UsedName::Normal(used_name) => {
                return Ref::Symbol(SymbolRef::new(
                  info.module,
                  direct_export.as_str().into(),
                  used_name[1..].to_vec(),
                  Arc::new(move |binding| normal_render(binding, as_call, call_context, asi_safe)),
                ));
              }
              UsedName::Inlined(inlined) => {
                return Ref::Inline(format!(
                  "/*{}*/{}",
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
          return Ref::Symbol(SymbolRef::new(
            info.module,
            raw_export.as_str().into(),
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
          FindTargetResult::NoValidTarget => {
            panic!(
              "Target module of reexport is not part of the concatenation (export '{:?}')",
              &export_id
            );
          }
          FindTargetResult::ValidTarget(reexport) => {
            if let Some(ref_info) = module_to_info_map.get(&reexport.module) {
              // https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/ConcatenatedModule.js#L457
              let build_meta = mg
                .module_by_identifier(&ref_info.id())
                .expect("should have module")
                .build_meta();
              return Self::get_binding(
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
            UsedName::Inlined(inlined) => Ref::Inline(inlined.render().into()),
          };
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
              info.name.clone().expect("should have symbol"),
              used_name,
              Arc::new(move |binding| normal_render(binding, as_call, call_context, asi_safe)),
            )),
            UsedName::Inlined(inlined) => {
              let comment = to_normal_comment(&format!(
                "inlined export {}",
                property_access(&export_name, 0)
              ));
              return Ref::Inline(format!("{}{}", inlined.render(), comment).into());
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
