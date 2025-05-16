use std::{collections::hash_map::Entry, sync::Arc};

use rayon::prelude::*;
use rspack_collections::{IdentifierIndexSet, IdentifierMap, UkeyIndexMap};
use rspack_core::{
  BoxModule, ChunkUkey, Compilation, ConcatenatedModule, ConcatenatedModuleIdent,
  ConcatenationScope, IdentCollector, ModuleInfo, NAMESPACE_OBJECT_EXPORT, RuntimeGlobals,
  SourceType, SpanExt, find_new_name,
  reserved_names::RESERVED_NAMES,
  rspack_sources::{ConcatSource, ReplaceSource, Source},
};
use rspack_error::Result;
use rspack_javascript_compiler::ast::Ast;
use rspack_plugin_javascript::{RenderSource, visitors::swc_visitor::resolver};
use rspack_util::{
  atom::Atom,
  fx_hash::{FxHashMap, FxHashSet, FxIndexSet},
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
  pub(crate) async fn render_chunk(
    &self,
    compilation: &Compilation,
    chunk_ukey: &ChunkUkey,
  ) -> Result<Option<RenderSource>> {
    let module_graph = compilation.get_module_graph();

    // modules that can be concatenated
    let mut concatenated_modules = Vec::new();
    let mut decl_modules: Vec<&BoxModule> = Vec::new();

    let mut concatenated_modules_map_by_compilation = self.concatenated_modules_map.lock().await;
    let concatenated_modules_map = concatenated_modules_map_by_compilation
      .get_mut(&compilation.id().0)
      .expect("should have map for compilation");

    let chunk_modules: IdentifierMap<&BoxModule> = compilation
      .chunk_graph
      .get_chunk_modules(chunk_ukey, &module_graph)
      .into_iter()
      .map(|m| (m.identifier(), m))
      .collect();

    for (id, m) in &chunk_modules {
      if let Some(ModuleInfo::Concatenated(_)) = concatenated_modules_map.get(id) {
        concatenated_modules.push(*m);
      } else {
        decl_modules.push(*m);
      }
    }

    concatenated_modules.sort_by(|m1, m2| {
      let m1_index = module_graph.get_post_order_index(&m1.identifier());
      let m2_index = module_graph.get_post_order_index(&m2.identifier());
      m1_index.cmp(&m2_index)
    });

    decl_modules.sort_by_key(|m| m.identifier());

    let mut imported_chunk: UkeyIndexMap<ChunkUkey, FxIndexSet<String>> = UkeyIndexMap::default();

    let mut runtime_requirements = RuntimeGlobals::empty();

    // find import
    let mut render_source = ConcatSource::default();
    let mut concate_infos: IdentifierMap<ConcateInfo> = concatenated_modules
      .par_iter()
      .filter_map(|m| {
        let identifier = m.identifier();
        let codegen_res = compilation.code_generation_results.get(&identifier, None);

        let Some(js_source) = codegen_res.get(&SourceType::JavaScript) else {
          return None;
        };

        let info = concatenated_modules_map.get(&identifier).unwrap();

        Some((identifier, info.as_concatenated().clone()))
      })
      .collect();

    let mut all_used_names: FxHashSet<Atom> =
      RESERVED_NAMES.iter().map(|s| Atom::new(*s)).collect();
    let mut top_level_declarations: FxHashSet<Atom> = FxHashSet::default();

    for m in &concatenated_modules {
      let info = concate_infos
        .get_mut(&m.identifier())
        .expect("should have info");

      let codegen_res = compilation
        .code_generation_results
        .get(&m.identifier(), None);

      for ((name, stxt), refs) in &info.binding_to_ref {
        if stxt == &info.global_ctxt {
          continue;
        }

        if all_used_names.insert(name.clone()) {
          info.internal_names.insert(name.clone(), name.clone());
        } else {
          let mut i = 1;
          let mut new_name: Atom = format!("{}${}", &name, i).into();
          while all_used_names.insert(new_name.clone()) {
            new_name = format!("{}${}", name, i).into();
            i += 1;
          }
          for ref_symbol in refs {
            info.source.unwrap().replace(
              ref_symbol.id.span.real_lo(),
              ref_symbol.id.span.real_hi(),
              &new_name,
              None,
            );
          }
          info.internal_names.insert(name.clone(), new_name);
        }

        // Handle the name passed through by namespace_export_symbol
        if let Some(ref namespace_export_symbol) = info.namespace_export_symbol {
          if namespace_export_symbol.starts_with(NAMESPACE_OBJECT_EXPORT)
            && namespace_export_symbol.len() > NAMESPACE_OBJECT_EXPORT.len()
          {
            let name =
              Atom::from(namespace_export_symbol[NAMESPACE_OBJECT_EXPORT.len()..].to_string());
            all_used_names.insert(name.clone());
            info
              .internal_names
              .insert(namespace_export_symbol.clone(), name.clone());
            top_level_declarations.insert(name.clone());
          }
        }

        // Handle namespaceObjectName for concatenated type
        let namespace_object_name =
          if let Some(ref namespace_export_symbol) = info.namespace_export_symbol {
            info.internal_names.get(namespace_export_symbol).cloned()
          } else {
            Some(find_new_name(
              "namespaceObject",
              &all_used_names,
              None,
              &m.readable_identifier(&compilation.options.context),
            ))
          };
        if let Some(namespace_object_name) = namespace_object_name {
          all_used_names.insert(namespace_object_name.clone());
          info.namespace_object_name = Some(namespace_object_name.clone());
          top_level_declarations.insert(namespace_object_name);
        }
      }
    }

    // Set with modules that need a generated namespace object
    let mut needed_namespace_objects = IdentifierIndexSet::default();

    // replace import specifier
    for m in &concatenated_modules {
      let info = concate_infos
        .get_mut(&m.identifier())
        .expect("should have info");
      for ident in &info.idents {
        if &ident.id.ctxt == &info.global_ctxt
          && let Some(match_module_ref) = ConcatenationScope::match_module_reference(&ident.id.sym)
        {
          let codegen_res = compilation
            .code_generation_results
            .get(&m.identifier(), None);
          let scope = codegen_res
            .concatenation_scope
            .as_ref()
            .expect("should have scope");

          let (ref_module, ref_info) = scope
            .modules_map
            .get_index(match_module_ref.index)
            .expect("should have module");

          let ref_module_instance = module_graph
            .module_by_identifier(ref_module)
            .expect("should have module");

          let final_name = ConcatenatedModule::get_final_name(
            &module_graph,
            ref_module,
            match_module_ref.ids,
            concatenated_modules_map,
            None,
            &mut needed_namespace_objects,
            match_module_ref.call,
            !match_module_ref.direct_import,
            ref_module_instance.build_info().strict,
            match_module_ref.asi_safe,
            &compilation.options.context,
          );
          let low = ident.id.span.real_lo();
          let high = ident.id.span.real_hi();
          info
            .source
            .unwrap()
            .replace(low, high + 2, &final_name, None);
        }
      }
    }

    // render import statement
    let link = compilation
      .chunk_graph
      .link
      .as_ref()
      .expect("should have chunk link");
    let link = link.get(chunk_ukey).expect("should have chunk link");

    let mut import_stmts = Vec::with_capacity(link.imports.len());
    for (chunk, imported) in &link.imports {
      if imported.is_empty() {
        import_stmts.push(format!(
          "import '__WEBPACK_CHUNK_UKEY_{}';\n",
          chunk.as_u32()
        ));
      } else {
        for (ref_module, import_atoms) in imported {
          let ref_info = concate_infos.get(ref_module).expect("should have info");

          let imported = import_atoms
            .iter()
            .map(|atom| {
              ref_info
                .internal_names
                .get(atom)
                .expect("internal name not found")
                .to_string()
            })
            .collect::<Vec<_>>();

          import_stmts.push(format!(
            "import {{{}}} from '__WEBPACK_CHUNK_UKEY_{}';\n",
            imported.join(", "),
            chunk.as_u32()
          ));
        }
      }
    }

    Ok(Some(RenderSource {
      source: Arc::new(render_source),
    }))
  }
}
