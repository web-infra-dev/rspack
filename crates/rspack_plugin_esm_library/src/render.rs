use std::{collections::hash_map::Entry, sync::Arc};

use rayon::prelude::*;
use rspack_collections::{IdentifierMap, UkeyIndexMap};
use rspack_core::{
  BoxModule, ChunkUkey, Compilation, ConcatenatedModuleIdent, ConcatenationScope, IdentCollector,
  RuntimeGlobals, SourceType, SpanExt,
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

#[derive(Debug)]
pub struct ConcateInfo {
  pub ast: Ast,
  pub all_used_names: FxHashSet<Atom>,
  pub internal_names: FxHashMap<Atom, Atom>,
  pub global_ctxt: SyntaxContext,
  pub module_ctxt: SyntaxContext,
  pub origin_source: Arc<dyn Source>,
  pub result_source: ReplaceSource<Arc<dyn Source>>,
  pub idents: Vec<ConcatenatedModuleIdent>,
  pub binding_to_ref: FxHashMap<(Atom, SyntaxContext), Vec<ConcatenatedModuleIdent>>,
}

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

    let concatenated_modules_map_by_compilation = self.concatenated_modules_map.lock().await;
    let concatenated_modules_map = concatenated_modules_map_by_compilation
      .get(&compilation.id().0)
      .expect("should have map for compilation");
    let chunk_modules: IdentifierMap<&BoxModule> = compilation
      .chunk_graph
      .get_chunk_modules(chunk_ukey, &module_graph)
      .into_iter()
      .map(|m| (m.identifier(), m))
      .collect();

    for (id, m) in &chunk_modules {
      if concatenated_modules_map.contains_key(id) {
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
        let replace_source = ReplaceSource::new(js_source.clone());
        let compiler = rspack_javascript_compiler::JavaScriptCompiler::new();
        let cm: Arc<swc_core::common::SourceMap> = Default::default();
        let readable_identifier = m.readable_identifier(&compilation.options.context);
        let fm = cm.new_source_file(
          Arc::new(FileName::Custom(readable_identifier.clone().into_owned())),
          js_source.source().clone().to_string(),
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
          identifier,
          ConcateInfo {
            ast,
            all_used_names,
            global_ctxt,
            module_ctxt,
            origin_source: js_source.clone(),
            internal_names: FxHashMap::default(),
            result_source: replace_source,
            idents,
            binding_to_ref,
          },
        ))
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
            info.result_source.replace(
              ref_symbol.id.span.real_lo(),
              ref_symbol.id.span.real_hi(),
              &new_name,
              None,
            );
          }
          info.internal_names.insert(name.clone(), new_name);
        }
      }
    }

    // replace cross module ident
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
        for (module, import_atom) in imported {}

        import_stmts.push(format!(
          "import {{{}}} from '__WEBPACK_CHUNK_UKEY_{}';\n",
          todo!(),
          chunk.as_u32()
        ));
      }
    }

    Ok(Some(RenderSource {
      source: Arc::new(render_source),
    }))
  }
}
