use std::{
  collections::hash_map::{Entry, OccupiedEntry},
  sync::Arc,
};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rspack_collections::{
  IdentifierIndexMap, IdentifierIndexSet, IdentifierMap, IdentifierSet, UkeyIndexMap, UkeyMap,
};
use rspack_core::{
  reserved_names::RESERVED_NAMES,
  rspack_sources::{ConcatSource, RawSource, ReplaceSource, Source},
  BoxModule, ChunkLink, ChunkUkey, Compilation, CompilationAfterCodeGeneration,
  CompilationAfterSeal, CompilationConcatenationScope, CompilationFinishModules, CompilationParams,
  CompilerCompilation, ConcatenatedModuleIdent, ConcatenatedModuleInfo, ConcatenationScope,
  DependencyId, ExportInfo, ExportInfoProvided, ExternalModuleInfo, IdentCollector, Module,
  ModuleGraph, ModuleGraphConnection, ModuleIdentifier, ModuleInfo, PathInfo, Plugin,
  RuntimeCondition, RuntimeGlobals, SourceType,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_javascript_compiler::ast::Ast;
use rspack_plugin_javascript::{
  visitors::swc_visitor::resolver, JavascriptModulesRenderChunkContent, JsPlugin, RenderSource,
};
use rspack_util::{
  atom::Atom,
  fx_hash::{FxHashMap, FxHashSet, FxIndexSet},
};
use swc_core::{
  common::{FileName, SyntaxContext},
  ecma::{
    ast::{EsVersion, Program},
    parser::{parse_file_as_module, Syntax},
  },
};
use tokio::sync::Mutex;

#[derive(Debug)]
struct ConcateInfo {
  pub ast: Ast,
  pub all_used_names: FxHashSet<Atom>,
  pub global_ctxt: SyntaxContext,
  pub module_ctxt: SyntaxContext,
  pub source: ReplaceSource<Arc<dyn Source>>,
  pub idents: Vec<ConcatenatedModuleIdent>,
  pub binding_to_ref: FxHashMap<(Atom, SyntaxContext), Vec<ConcatenatedModuleIdent>>,
}

#[plugin]
#[derive(Debug, Default)]
pub struct EsmLibraryPlugin {
  pub concatenated_modules_map: Mutex<FxHashMap<u32, Arc<IdentifierIndexMap<ModuleInfo>>>>,
}

#[plugin_hook(CompilerCompilation for EsmLibraryPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let mut hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  hooks
    .render_chunk_content
    .tap(render_chunk_content::new(self));
  Ok(())
}

#[plugin_hook(JavascriptModulesRenderChunkContent for EsmLibraryPlugin)]
async fn render_chunk_content(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
) -> Result<Option<RenderSource>> {
  let module_graph = compilation.get_module_graph();

  // modules that can be concatenated
  let mut concatenated_modules = Vec::new();
  let mut decl_modules: Vec<&Box<dyn Module>> = Vec::new();

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
          source: replace_source,
          idents,
          binding_to_ref,
        },
      ))
    })
    .collect();

  let mut all_used_names: FxHashSet<Atom> = RESERVED_NAMES.iter().map(|s| Atom::new(*s)).collect();
  let mut top_level_declarations: FxHashSet<Atom> = FxHashSet::default();

  // Replace global ident
  for m in &concatenated_modules {
    let info = concate_infos
      .get(&m.identifier())
      .expect("should have info");
    let mut source = info.source.clone();

    let codegen_res = compilation
      .code_generation_results
      .get(&m.identifier(), None);

    // replace module ref placeholder with normal ident or import symbol
    if let Some(scope) = &codegen_res.concatenation_scope {}
  }

  Ok(Some(RenderSource {
    source: Arc::new(render_source),
  }))
}

#[plugin_hook(CompilationAfterSeal for EsmLibraryPlugin)]
async fn after_seal(&self, compilation: &mut Compilation) -> Result<()> {
  self
    .concatenated_modules_map
    .lock()
    .await
    .remove(&compilation.id().0);
  Ok(())
}

#[plugin_hook(CompilationFinishModules for EsmLibraryPlugin)]
async fn finish_modules(&self, compilation: &mut Compilation) -> Result<()> {
  let module_graph = compilation.get_module_graph();
  let mut modules_map = IdentifierIndexMap::default();
  let modules = module_graph.modules();
  let mut modules = modules.iter().collect::<Vec<_>>();
  modules.sort_by(|(m1, _), (m2, _)| m1.cmp(m2));

  for (idx, (module_identifier, module)) in modules.into_iter().enumerate() {
    // make sure all exports are provided
    let exports_info = module_graph.get_exports_info(&module_identifier);

    let mut should_scope_hoisting = true;
    if module.as_normal_module().is_none() {
      should_scope_hoisting = false;
    };
    if should_scope_hoisting {
      for export_info in exports_info.exports(&module_graph) {
        if !matches!(
          export_info.provided(&module_graph),
          Some(ExportInfoProvided::True)
        ) {
          should_scope_hoisting = false;
          break;
        };

        if export_info.is_reexport(&module_graph) && export_info.get_target(&module_graph).is_none()
        {
          should_scope_hoisting = false;
          break;
        }
      }
    }

    if should_scope_hoisting {
      modules_map.insert(
        *module_identifier,
        ModuleInfo::Concatenated(Box::new(ConcatenatedModuleInfo {
          index: idx,
          module: *module_identifier,
          ..Default::default()
        })),
      );
    } else {
      modules_map.insert(
        *module_identifier,
        ModuleInfo::External(ExternalModuleInfo {
          index: idx,
          module: *module_identifier,
          runtime_condition: if exports_info.is_used(&module_graph, None) {
            RuntimeCondition::Boolean(true)
          } else {
            RuntimeCondition::Boolean(false)
          },
          interop_namespace_object_used: false,
          interop_namespace_object_name: None,
          interop_namespace_object2_used: false,
          interop_namespace_object2_name: None,
          interop_default_access_used: false,
          interop_default_access_name: None,
          name: None,
        }),
      );
    }
  }

  let id = compilation.id();

  let mut self_modules_map = self.concatenated_modules_map.lock().await;
  self_modules_map.insert(id.0, Arc::new(modules_map));

  Ok(())
}

impl EsmLibraryPlugin {
  fn get_imports(
    m: &Box<dyn Module>,
    module_graph: &ModuleGraph,
  ) -> impl Iterator<Item = (ModuleIdentifier, Vec<DependencyId>)> {
    let mut modules = IdentifierIndexMap::default();
    for dep in m.get_dependencies() {
      let Some(conn) = module_graph.connection_by_dependency_id(dep) else {
        continue;
      };

      if !conn.is_target_active(module_graph, None) {
        continue;
      }

      let Some(dep_module) = module_graph.module_identifier_by_dependency_id(dep) else {
        continue;
      };

      let connections: &mut Vec<DependencyId> = modules.entry(*dep_module).or_default();
      connections.push(conn.dependency_id);
    }

    modules.into_iter()
  }
}

#[plugin_hook(CompilationConcatenationScope for EsmLibraryPlugin)]
async fn concatenation_scope(
  &self,
  compilation: &Compilation,
  module: ModuleIdentifier,
) -> Result<Option<ConcatenationScope>> {
  let modules_map = self.concatenated_modules_map.lock().await;
  let modules_map = modules_map
    .get(&compilation.id().0)
    .expect("should has compilation");

  let Some(current_module) = modules_map.get(&module) else {
    return Ok(None);
  };
  let current_module = current_module.as_concatenated().clone();

  Ok(Some(ConcatenationScope::new(
    modules_map.clone(),
    current_module,
  )))
}

#[plugin_hook(CompilationAfterCodeGeneration for EsmLibraryPlugin)]
async fn after_code_generation(&self, compilation: &mut Compilation) -> Result<()> {
  let module_graph = compilation.get_module_graph();
  let all_chunks: Vec<ChunkUkey> = compilation.chunk_by_ukey.keys().copied().collect();
  let concate_modules_map = self.concatenated_modules_map.lock().await;
  let concate_modules_map = concate_modules_map
    .get(&compilation.id().0)
    .expect("should has compilation");
  let mut link = UkeyMap::<ChunkUkey, ChunkLink>::default();
  let mut record_exports = UkeyMap::<ChunkUkey, IdentifierMap<FxHashSet<Atom>>>::default();

  // calculate imports to other chunks
  for chunk_ukey in all_chunks {
    link.entry(chunk_ukey).or_default();

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
      let imports = chunk_link.imports.entry(chunk_ukey).or_default();

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

  compilation.chunk_graph.link = Some(link);
  Ok(())
}

impl Plugin for EsmLibraryPlugin {
  fn apply(
    &self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
    _options: &rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));

    ctx
      .context
      .compilation_hooks
      .finish_modules
      .tap(finish_modules::new(self));

    ctx
      .context
      .compilation_hooks
      .after_code_generation
      .tap(after_code_generation::new(self));

    ctx
      .context
      .compilation_hooks
      .concatenation_scope
      .tap(concatenation_scope::new(self));

    ctx
      .context
      .compilation_hooks
      .after_seal
      .tap(after_seal::new(self));

    Ok(())
  }
}
