use std::{collections::hash_map::Entry, sync::Arc};

use rayon::prelude::*;
use rspack_collections::{
  IdentifierIndexMap, IdentifierIndexSet, IdentifierMap, IdentifierSet, UkeyMap,
};
use rspack_core::{
  Binding, BoxModule, BuildMetaDefaultObject, BuildMetaExportsType, ChunkLink, ChunkUkey,
  Compilation, ConcatenatedModule, ConcatenatedModuleIdent, ConcatenatedModuleInfo,
  ConcatenationScope, IdentCollector, ModuleIdentifier, ModuleInfo, SourceType, find_new_name,
  reserved_names::RESERVED_NAMES, rspack_sources::ReplaceSource,
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
    parser::{Syntax, parse_file_as_module},
  },
};

use crate::EsmLibraryPlugin;

impl EsmLibraryPlugin {
  pub(crate) async fn calculate_chunk_relation(&self, compilation: &mut Compilation) -> Result<()> {
    let module_graph = compilation.get_module_graph();
    let all_chunks: Vec<ChunkUkey> = compilation.chunk_by_ukey.keys().copied().collect();
    let mut concate_modules_map = self.concatenated_modules_map.lock().await;
    let concate_modules_map = Arc::get_mut(
      concate_modules_map
        .get_mut(&compilation.id().0)
        .expect("should has compilation"),
    )
    .unwrap();
    let mut link = UkeyMap::<ChunkUkey, ChunkLink>::default();
    let mut record_exports = UkeyMap::<ChunkUkey, IdentifierMap<FxHashSet<Atom>>>::default();

    // calculate imports to other chunks
    for chunk_ukey in &all_chunks {
      link.entry(*chunk_ukey).or_default();

      let all_chunk_concate_modules = compilation
        .chunk_graph
        .get_chunk_modules_identifier(&chunk_ukey)
        .iter()
        .filter(|m| {
          concate_modules_map
            .get(*m)
            .expect("should have module")
            .try_as_concatenated()
            .is_some()
        })
        .copied()
        .collect::<IdentifierSet>();

      let mut concate_chunk_modules = all_chunk_concate_modules
        .iter()
        .copied()
        .collect::<Vec<_>>();

      concate_chunk_modules.sort_by(|m1, m2| {
        let m1_index = module_graph.get_post_order_index(m1);
        let m2_index = module_graph.get_post_order_index(m2);
        m1_index.cmp(&m2_index)
      });

      let chunk_link = link.get_mut(&chunk_ukey).expect("should have chunk link");
      chunk_link.hoisted_modules = concate_chunk_modules.iter().copied().collect();
      let mut errors = vec![];

      for m in &concate_chunk_modules {
        let module = module_graph
          .module_by_identifier(&m)
          .expect("should have module");
        let codegen_res = compilation.code_generation_results.get(&m, None);
        let Some(concatenation_scope) = &codegen_res.concatenation_scope else {
          continue;
        };
        let imports = chunk_link.imports.entry(*chunk_ukey).or_default();

        for (imported, refs) in &concatenation_scope.refs {
          if compilation
            .chunk_graph
            .is_module_in_chunk(imported, *chunk_ukey)
          {
            continue;
          }

          let import_refs = imports.entry(*imported).or_default();

          let chunk = compilation.chunk_graph.get_module_chunks(*imported);
          if chunk.len() > 1 {
            errors.push(format!("module exist in multiple chunks {}", imported));
            continue;
          }

          if chunk.is_empty() {
            errors.push(format!("module not exist in any chunk {}", imported));
            continue;
          }

          let chunk_ukey = chunk
            .into_iter()
            .next()
            .expect("should have at least one chunk");
          let exports = record_exports.entry(*chunk_ukey).or_default();
          let exports = exports.entry(*imported).or_default();

          let imported_exports_info = module_graph.get_exports_info(imported);

          for match_ref in refs.values() {
            let imported_name = &match_ref.ids[0];
            import_refs.insert(imported_name.clone());
            exports.insert(imported_name.clone());
          }
        }
      }
    }

    // record exports
    for (chunk_ukey, exports) in record_exports {
      let chunk_link = link.entry(chunk_ukey).or_default();
      chunk_link.exports = exports;
    }

    concate_modules_map
      .par_iter_mut()
      .for_each(|(id, info)| match info {
        rspack_core::ModuleInfo::External(external_module_info) => {}
        rspack_core::ModuleInfo::Concatenated(concate_info) => {
          let codegen_res = compilation.code_generation_results.get(id, None);
          let Some(js_source) = codegen_res.get(&SourceType::JavaScript) else {
            return;
          };

          *concate_info = Box::new(
            codegen_res
              .concatenation_scope
              .as_ref()
              .unwrap()
              .current_module
              .clone(),
          );

          let m = module_graph
            .module_by_identifier(id)
            .expect("should have module");
          let replace_source = ReplaceSource::new(js_source.clone());
          let compiler = rspack_javascript_compiler::JavaScriptCompiler::new();
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
        }
      });

    concate_modules_map.iter_mut().for_each(|(id, info)| {
      let module: &BoxModule = module_graph
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
        let external_name_interop: Atom = format!("{}_default", readable_identifier).into();
        info.set_interop_default_access_name(Some(external_name_interop.clone()));
      }
    });

    compilation.chunk_graph.link = Some(link);
    Ok(())
  }

  pub(crate) async fn link(&self, compilation: &mut Compilation) -> Result<()> {
    let mut compilation_modules_map = self.concatenated_modules_map.lock().await;
    let mut modules_map = compilation_modules_map
      .get_mut(&compilation.id().0)
      .unwrap();
    let mut link = std::mem::take(&mut compilation.chunk_graph.link).unwrap();
    let modules_map = Arc::get_mut(&mut modules_map).unwrap();
    let module_graph = compilation.get_module_graph();

    let mut chunk_used_names = UkeyMap::default();
    // for each chunk
    for (chunk_ukey, chunk_link) in &mut link {
      let mut all_used_names: FxHashSet<Atom> = RESERVED_NAMES
        .iter()
        .map(|s| Atom::new(*s))
        .chain(chunk_link.hoisted_modules.iter().flat_map(|m| {
          let info = modules_map.get(m).unwrap();
          info
            .as_concatenated()
            .global_scope_ident
            .iter()
            .map(|ident| ident.id.sym.clone())
        }))
        .collect();
      let mut top_level_declarations = FxHashSet::default();

      for id in &chunk_link.hoisted_modules {
        let concate_info = modules_map.get_mut(id).unwrap().as_concatenated();
        all_used_names.extend(concate_info.all_used_names.clone());
      }

      // deconflict top level symbols
      for id in &chunk_link.hoisted_modules {
        let module = module_graph.module_by_identifier(&id).unwrap();
        let exports_type = module.build_meta().exports_type;
        let default_object = module.build_meta().default_object;
        let readable_identifier = module.readable_identifier(&compilation.options.context);
        let info = modules_map.get_mut(id).unwrap();

        let concate_info = info.as_concatenated_mut();

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
          top_level_declarations.insert(external_name_interop.clone());
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
          top_level_declarations.insert(external_name_interop.clone());
        }

        if matches!(
          exports_type,
          BuildMetaExportsType::Dynamic | BuildMetaExportsType::Unset
        ) {
          let external_name_interop: Atom =
            find_new_name("default", &all_used_names, None, &readable_identifier);
          all_used_names.insert(external_name_interop.clone());
          info.set_interop_default_access_name(Some(external_name_interop.clone()));
          top_level_declarations.insert(external_name_interop.clone());
        }

        let codegen_res = compilation.code_generation_results.get(&id, None);
        let concatenation_scope = codegen_res.concatenation_scope.as_ref().unwrap();

        let mut internal_names = FxHashMap::default();
        let concate_info = info.as_concatenated();

        for (atom, ctxt) in concate_info.binding_to_ref.keys() {
          if ctxt != &concate_info.module_ctxt {
            continue;
          }

          if all_used_names.contains(atom) {
            let new_name = find_new_name(&atom, &all_used_names, None, &readable_identifier);
            all_used_names.insert(new_name.clone());
            internal_names.insert(atom.clone(), new_name);
          } else {
            all_used_names.insert(atom.clone());
            internal_names.insert(atom.clone(), atom.clone());
          }
        }

        let concate_info = info.as_concatenated_mut();
        concate_info.internal_names = internal_names;
      }

      for external_module in compilation
        .chunk_graph
        .get_chunk_modules(chunk_ukey, &module_graph)
        .into_iter()
        .filter(|m| !chunk_link.hoisted_modules.contains(&m.identifier()))
      {
        let ModuleInfo::External(info) =
          modules_map.get_mut(&external_module.identifier()).unwrap()
        else {
          unreachable!("should be external module");
        };
        let name = find_new_name(
          "",
          &all_used_names,
          None,
          &external_module.readable_identifier(&compilation.options.context),
        );
        info.name = Some(name.clone());
        all_used_names.insert(name.clone());
        top_level_declarations.insert(name.clone());
      }

      chunk_used_names.insert(*chunk_ukey, all_used_names);
    }

    // modify cross module references
    let mut exports = UkeyMap::<ChunkUkey, IdentifierMap<FxHashSet<Atom>>>::default();
    for (chunk, chunk_link) in &mut link {
      let all_used_names = chunk_used_names
        .get_mut(chunk)
        .expect("should have all_used_names");
      let mut ref_to_final_name = FxHashMap::default();

      let mut needed_namespace_objects = IdentifierIndexSet::default();

      for m in chunk_link.hoisted_modules.clone() {
        let codegen_res = compilation.code_generation_results.get(&m, None);
        let concatenation_scope = codegen_res.concatenation_scope.as_ref().unwrap();

        for (ref_module, refs) in &concatenation_scope.refs {
          for (ref_string, options) in refs.iter() {
            if ref_to_final_name.contains_key(ref_string) {
              continue;
            }

            // if imported specifier is in the same chunk
            // the final name is symbol in current chunk
            // if imported specifier is in other chunk
            // the final name is symbol in that chunk

            if !compilation
              .chunk_graph
              .is_module_in_chunk(ref_module, *chunk)
            {
              // `get_final_name()` assume the symbol is in the same chunk,
              // and use `info.internal_names` to get the deconflicted symbol,
              // the internal_names is generated based on local symbols in chunk
              // but the module may in other chunks, so this assumption is wrong,
              // we need to deconflict the symbol again
              let mut binding = ConcatenatedModule::get_final_binding(
                &module_graph,
                ref_module,
                options.ids.clone(),
                modules_map,
                None,
                &mut needed_namespace_objects,
                options.call,
                !options.direct_import,
                options.asi_safe,
                &mut Default::default(),
              );

              let module_id = binding.identifier();
              let ref_chunk = Self::get_module_chunk(module_id, compilation);

              if let Binding::Symbol(symbol_binding) = &binding {
                exports
                  .entry(ref_chunk)
                  .or_default()
                  .entry(module_id)
                  .or_default()
                  .insert(symbol_binding.name.clone());
              }
              ref_to_final_name.insert(
                ref_string.strip_suffix("._").unwrap().to_string(),
                rspack_core::ModuleReference::Binding(binding),
              );
            } else {
              let final_name = ConcatenatedModule::get_final_name(
                &module_graph,
                ref_module,
                options.ids.clone(),
                modules_map,
                None,
                &mut needed_namespace_objects,
                options.call,
                !options.direct_import,
                true,
                options.asi_safe,
                &compilation.options.context,
              );
              ref_to_final_name.insert(
                ref_string.strip_suffix("._").unwrap().to_string(),
                rspack_core::ModuleReference::Str(final_name),
              );
            }
          }
        }
      }

      chunk_link.needed_namespace_objects = needed_namespace_objects;
      chunk_link.ref_to_final_name = ref_to_final_name;
    }

    for (chunk, exports) in exports {
      link.entry(chunk).or_default().exports = exports;
    }
    compilation.chunk_graph.link = Some(link);

    Ok(())
  }

  pub fn get_module_chunk(m: ModuleIdentifier, compilation: &Compilation) -> ChunkUkey {
    let chunks = compilation.chunk_graph.get_module_chunks(m);
    if chunks.is_empty() {
      unimplemented!("module is not in any chunk");
    }

    if chunks.len() > 1 {
      unimplemented!("module is in multiple chunks");
    }

    chunks.into_iter().next().unwrap().clone()
  }
}
