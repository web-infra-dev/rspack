use std::{collections::hash_map::Entry, sync::Arc};

use rayon::prelude::*;
use rspack_collections::{IdentifierIndexMap, IdentifierIndexSet, IdentifierMap, UkeyMap};
use rspack_core::{
  find_new_name, property_name, reserved_names::RESERVED_NAMES, returning_function,
  rspack_sources::ReplaceSource, Binding, BuildMetaDefaultObject, BuildMetaExportsType,
  ChunkLinkContext, ChunkUkey, Compilation, ConcatenatedModule, ConcatenatedModuleIdent,
  ExportInfoGetter, ExportProvided, IdentCollector, ModuleIdentifier, ModuleInfo, RuntimeGlobals,
  SourceType, UsedNameItem, NAMESPACE_OBJECT_EXPORT,
};
use rspack_error::Result;
use rspack_javascript_compiler::ast::Ast;
use rspack_plugin_javascript::visitors::swc_visitor::resolver;
use rspack_util::{
  atom::Atom,
  fx_hash::{FxHashMap, FxHashSet},
};
use swc_core::{
  common::{FileName, SyntaxContext},
  ecma::{
    ast::{EsVersion, Program},
    parser::{parse_file_as_module, Syntax},
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

    // analyze every modules and collect identifiers to concate_modules_map
    self.analyze_module(compilation, concate_modules_map);

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
              vec![export_info
                .as_data(&module_graph)
                .name()
                .cloned()
                .unwrap_or("".into())],
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
      unimplemented!("module is not in any chunk");
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

  fn analyze_module(
    &self,
    compilation: &Compilation,
    concate_modules_map: &mut IdentifierIndexMap<ModuleInfo>,
  ) {
    let module_graph = compilation.get_module_graph();
    // analyze each module and collect all identifiers
    concate_modules_map
      .par_iter_mut()
      .filter(|(m, _)| !Self::is_orphan(**m, compilation))
      .for_each(|(id, info)| match info {
        rspack_core::ModuleInfo::External(external_module_info) => {
          // we use Object.assign(__webpack_require__.m, {...}) to register modules
          external_module_info
            .runtime_requirements
            .insert(RuntimeGlobals::REQUIRE | RuntimeGlobals::MODULE_FACTORIES);
        }
        rspack_core::ModuleInfo::Concatenated(concate_info) => {
          let codegen_res = compilation.code_generation_results.get(id, None);
          let Some(js_source) = codegen_res.get(&SourceType::JavaScript) else {
            return;
          };

          *concate_info = Box::new(
            codegen_res
              .concatenation_scope
              .as_ref()
              .expect("should have concatenation scope")
              .current_module
              .clone(),
          );

          let m = module_graph
            .module_by_identifier(id)
            .expect("should have module");
          let cm: Arc<swc_core::common::SourceMap> = Default::default();
          let readable_identifier = m.readable_identifier(&compilation.options.context);
          let fm = cm.new_source_file(
            Arc::new(FileName::Custom(readable_identifier.clone().into_owned())),
            js_source.source().to_string(),
          );
          let mut errors = vec![];
          let module =
            parse_file_as_module(&fm, Syntax::default(), EsVersion::EsNext, None, &mut errors)
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

          let mut binding_to_ref: FxHashMap<(Atom, SyntaxContext), Vec<ConcatenatedModuleIdent>> =
            FxHashMap::default();

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
          concate_info.ast = Some(ast);
          concate_info.source = Some(ReplaceSource::new(js_source.clone()));
          concate_info.internal_source = Some(js_source.clone());
          concate_info.runtime_requirements = codegen_res.runtime_requirements;
        }
      });

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

    for (chunk, chunk_link) in link.iter_mut() {
      let mut ref_to_final_name = FxHashMap::default();

      for m in chunk_link.hoisted_modules.clone() {
        let codegen_res = compilation.code_generation_results.get(&m, None);
        let concatenation_scope = codegen_res
          .concatenation_scope
          .as_ref()
          .expect("should have concatenation scope for scope hoisted module");

        for (ref_module, all_refs) in &concatenation_scope.refs {
          // import all atoms from ref_module
          for (ref_string, options) in all_refs.iter() {
            if ref_to_final_name.contains_key(ref_string) {
              continue;
            }

            // if imported specifier is in the same chunk
            // the final name is symbol in current chunk

            // if imported specifier is in other chunk
            // the final name is symbol in referenced chunk
            let binding = ConcatenatedModule::get_final_binding(
              &module_graph,
              &compilation.module_graph_cache_artifact,
              ref_module,
              options.ids.clone(),
              concate_modules_map,
              None,
              needed_namespace_objects,
              options.call,
              !options.direct_import,
              options.asi_safe,
              &mut Default::default(),
            );

            let module_id = binding.identifier();
            let ref_chunk = Self::get_module_chunk(module_id, compilation);
            match &binding {
              Binding::Raw(_raw_binding) => {
                // import to non-scope-hoisted module or namespace name
              }
              Binding::Symbol(symbol_binding) => {
                if &ref_chunk != chunk {
                  // ref chunk should expose the symbol
                  exports
                    .entry(ref_chunk)
                    .or_default()
                    .entry(module_id)
                    .or_default()
                    .insert(symbol_binding.name.clone());
                }
              }
            }

            ref_to_final_name.insert(
              ref_string
                .strip_suffix("._")
                .expect("should have prefix: '._'")
                .to_string(),
              rspack_core::ModuleReference::Binding(binding),
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

      chunk_link.needed_namespace_objects = needed_namespace_objects.clone();
      chunk_link.ref_to_final_name = ref_to_final_name;
    }

    for (chunk, exports) in exports {
      link.entry(chunk).or_default().exports = exports;
    }
  }
}
