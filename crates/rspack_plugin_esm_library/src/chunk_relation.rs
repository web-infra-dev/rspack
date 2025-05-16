use std::sync::Arc;

use rayon::prelude::*;
use rspack_collections::{IdentifierMap, IdentifierSet, UkeyMap};
use rspack_core::{
  ChunkLink, ChunkUkey, Compilation, ConcatenatedModuleInfo, ConcatenationScope, IdentCollector,
  SourceType,
};
use rspack_error::Result;
use rspack_javascript_compiler::ast::Ast;
use rspack_plugin_javascript::visitors::swc_visitor::resolver;
use rspack_util::{
  atom::Atom,
  fx_hash::{FxHashMap, FxHashSet},
};
use swc_core::{common::SyntaxContext, ecma::parser::parse_file_as_module};

use crate::EsmLibraryPlugin;

impl EsmLibraryPlugin {
  pub(crate) async fn calculate_chunk_relation(&self, compilation: &mut Compilation) -> Result<()> {
    let module_graph = compilation.get_module_graph();
    let all_chunks: Vec<ChunkUkey> = compilation.chunk_by_ukey.keys().copied().collect();
    let concate_modules_map = self.concatenated_modules_map.lock().await;
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

      let all_chunk_modules = compilation
        .chunk_graph
        .get_chunk_modules_identifier(&chunk_ukey)
        .iter()
        .filter(|m| concate_modules_map.contains_key(*m))
        .copied()
        .collect::<IdentifierSet>();

      let mut chunk_modules = all_chunk_modules.iter().copied().collect::<Vec<_>>();

      chunk_modules.sort_by(|m1, m2| {
        let m1_index = module_graph.get_post_order_index(m1);
        let m2_index = module_graph.get_post_order_index(m2);
        m1_index.cmp(&m2_index)
      });

      let chunk_link = link.get_mut(&chunk_ukey).expect("should have chunk link");
      let mut errors = vec![];

      for m in chunk_modules {
        let module = module_graph
          .module_by_identifier(&m)
          .expect("should have module");
        let codegen_res = compilation.code_generation_results.get(&m, None);
        let Some(concatenation_scope) = &codegen_res.concatenation_scope else {
          continue;
        };
        let imports = chunk_link.imports.entry(*chunk_ukey).or_default();

        for (imported, refs) in &concatenation_scope.refs {
          if all_chunk_modules.contains(imported) {
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

          for import_ref in refs {
            let match_ref = ConcatenationScope::match_module_reference(&import_ref)
              .expect("should have exact match");

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
      .par_iter()
      .filter_map(|(id, info)| {
        if let Some(concate_info) = info.try_as_concatenated() {
          let codegen_res = compilation.code_generation_results.get(id, None);
          let Some(js_source) = codegen_res.get(&SourceType::JavaScript) else {
            return None;
          };

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

          Some((
            id,
            ConcatenatedModuleInfo {
              ast: Some(ast),
              global_ctxt,
              module_ctxt,
              idents,
              binding_to_ref,
              ..concate_info.clone()
            },
          ))
        } else {
          None
        }
      })
      .collect();

    compilation.chunk_graph.link = Some(link);
    Ok(())
  }
}
