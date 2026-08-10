mod module_references;
mod runtime_mode;

use std::{borrow::Cow, sync::Arc};

use rspack_collections::{IdentifierIndexMap, IdentifierIndexSet};
use rspack_core::{
  AssetInfo, Chunk, ChunkGraph, ChunkGroup, ChunkRenderContext, ChunkUkey,
  CodeGenerationModuleReferences, Compilation, ConcatenatedModuleInfo, InitFragment,
  ModuleIdentifier, ModuleInfo, PathData, PathInfo, RuntimeCodeTemplate, RuntimeGlobals,
  RuntimeRequirementsDependency, SourceType, export_name, get_js_chunk_filename_template,
  get_undo_path, is_esm_dep_like, render_init_fragments,
  rspack_sources::{ConcatSource, RawStringSource, ReplaceSource, Source, SourceExt},
};
use rspack_error::Result;
use rspack_plugin_javascript::{
  JsPlugin, RenderSource,
  runtime::{AUTO_PUBLIC_PATH_PLACEHOLDER, render_module, render_runtime_modules},
  url_plugin::replace_static_url_placeholders,
};
use rspack_util::{
  SpanExt,
  atom::Atom,
  fx_hash::{FxHashMap, FxHashSet, FxIndexMap, FxIndexSet},
};

use self::{
  module_references::relocate_module_references,
  runtime_mode::{RuntimeImportRenderContext, RuntimeRenderContext, renderer_for},
};
use crate::{
  chunk_link::{ChunkLinkContext, CjsWrapperPlan, ReExportFrom, Ref},
  initializer::render_initializer,
  is_css_only_module,
  plugin::RSPACK_ESM_RUNTIME_CHUNK,
  runtime::without_module_loader_runtime_globals,
};

/// `RuntimeGlobals::REQUIRE` traditionally meant both "call the module-id
/// dispatcher" and "access the runtime scope object". Modern-module removes
/// the former, but user code such as `__webpack_require__.custom` still needs
/// the latter. The API parser records that access as an explicit
/// `REQUIRE_SCOPE` presentational dependency, which lets this renderer keep
/// the object capability without reintroducing a dispatcher.
fn chunk_uses_explicit_runtime_scope(compilation: &Compilation, chunk_ukey: &ChunkUkey) -> bool {
  compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .get_chunk_modules_identifier(chunk_ukey)
    .iter()
    .filter_map(|identifier| {
      compilation
        .get_module_graph()
        .module_by_identifier(identifier)
    })
    .filter_map(|module| module.get_presentational_dependencies())
    .flatten()
    .filter_map(|dependency| {
      dependency
        .as_any()
        .downcast_ref::<RuntimeRequirementsDependency>()
    })
    .any(|dependency| {
      dependency
        .runtime_requirements
        .contains(RuntimeGlobals::REQUIRE_SCOPE)
    })
}

fn render_cjs_wrapper(plan: &CjsWrapperPlan) -> RawStringSource {
  let helper = &plan.helper;
  let invoke_factory = "factory.call(module.exports, module, module.exports);";
  let source = if plan.strict_error_handling {
    format!(
      "var {helper} = (factory, module) => function __require() {{\n\
       \tif (module !== undefined) {{\n\
       \t\tif (module.error !== undefined) throw module.error;\n\
       \t\treturn module.exports;\n\
       \t}}\n\
       \tmodule = {{ exports: {{}} }};\n\
       \ttry {{\n\
       \t\t{invoke_factory}\n\
       \t}} catch (error) {{\n\
       \t\tmodule.error = error;\n\
       \t\tthrow error;\n\
       \t}}\n\
       \treturn module.exports;\n\
       }};\n"
    )
  } else {
    format!(
      "var {helper} = (factory, module) => function __require() {{\n\
       \tif (module === undefined) {{\n\
       \t\tmodule = {{ exports: {{}} }};\n\
       \t\t{invoke_factory}\n\
       \t}}\n\
       \treturn module.exports;\n\
       }};\n"
    )
  };
  RawStringSource::from(source)
}

fn render_esm_wrapper(chunk_link: &ChunkLinkContext) -> RawStringSource {
  let helper = &chunk_link
    .wrapped_runtime
    .esm
    .as_ref()
    .expect("chunk with scope-hoisted initializers should have an ESM wrapper")
    .helper;
  RawStringSource::from(format!(
    "var {helper} = (fn, result) => function __init() {{\n\
     \tif (fn) {{\n\
     \t\tvar value = fn(fn = 0);\n\
     \t\tresult = value && typeof value.then === \"function\" ? value.then(value => result = value) : value;\n\
     \t}}\n\
     \treturn result;\n\
     }};\n"
  ))
}

fn render_static_initializer_dependencies(
  module_identifier: ModuleIdentifier,
  compilation: &Compilation,
  chunk_link: &ChunkLinkContext,
  module_infos: &IdentifierIndexMap<ModuleInfo>,
) -> String {
  let module_graph = compilation.get_module_graph();
  let module = module_graph
    .module_by_identifier(&module_identifier)
    .expect("initializer-backed module should exist");
  let current_is_async =
    rspack_core::ModuleGraph::is_async(&compilation.async_modules_artifact, &module_identifier);
  let mut targets = IdentifierIndexSet::default();
  for dependency_id in module.get_dependencies() {
    let dependency = module_graph.dependency_by_id(dependency_id);
    if !is_esm_dep_like(dependency) || dependency.get_phase().is_defer() {
      continue;
    }
    let Some(connection) = module_graph.connection_by_dependency_id(dependency_id) else {
      continue;
    };
    if !connection.is_target_active(
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
    targets.insert(*connection.module_identifier());
  }

  let mut source = String::new();
  let mut uses_async_dependency_temp = false;
  for target in targets {
    if !matches!(
      module_infos.get(&target),
      Some(ModuleInfo::Concatenated(info)) if info.initializer.is_some()
    ) {
      continue;
    }
    let Some(initializer) = chunk_link.module_initializers.get(&target) else {
      continue;
    };
    let target_is_async =
      rspack_core::ModuleGraph::is_async(&compilation.async_modules_artifact, &target);
    if current_is_async && target_is_async {
      uses_async_dependency_temp = true;
      let temp = chunk_link
        .async_dependency_temp
        .as_ref()
        .expect("async initializer should have a deconflicted dependency scratch binding");
      source.push_str(&format!(
        "\tif (({temp} = {initializer}()) && typeof {temp}.then === \"function\") await {temp};\n"
      ));
    } else {
      source.push('\t');
      source.push_str(initializer);
      source.push_str("();\n");
    }
  }
  if uses_async_dependency_temp {
    source.insert_str(
      0,
      &format!(
        "\tvar {};\n",
        chunk_link
          .async_dependency_temp
          .as_ref()
          .expect("async initializer should have a dependency scratch binding")
      ),
    );
  }
  source
}

#[inline]
fn get_chunk(compilation: &Compilation, chunk_ukey: ChunkUkey) -> &Chunk {
  compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(&chunk_ukey)
}

use crate::EsmLibraryPlugin;

impl EsmLibraryPlugin {
  fn get_entrypoint(chunk_ukey: ChunkUkey, compilation: &Compilation) -> Option<&ChunkGroup> {
    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(&chunk_ukey);

    // A chunk can be both an entry chunk and a dynamic-import target of
    // another entry. Prefer the entrypoint owned by this chunk before walking
    // parent groups. Otherwise runtime lookup can select the importing entry's
    // runtime while this chunk still renders its own entry runtime, causing
    // imported runtime bindings to collide with local helper declarations.
    for group_ukey in chunk.groups() {
      let group = compilation
        .build_chunk_graph_artifact
        .chunk_group_by_ukey
        .expect_get(group_ukey);
      if group.kind.is_entrypoint() && group.get_entrypoint_chunk() == chunk_ukey {
        return Some(group);
      }
    }

    let group = chunk.groups().iter().next()?;
    let group = compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .expect_get(group);
    let mut stack = vec![group];
    let mut visited = FxHashSet::default();

    while let Some(group) = stack.pop() {
      if !visited.insert(group.ukey) {
        continue;
      }

      if group.kind.is_entrypoint() {
        return Some(group);
      }

      stack.extend(group.parents_iterable().map(|group| {
        compilation
          .build_chunk_graph_artifact
          .chunk_group_by_ukey
          .expect_get(group)
      }));
    }

    None
  }

  pub(crate) fn get_runtime_chunk(chunk_ukey: ChunkUkey, compilation: &Compilation) -> ChunkUkey {
    Self::get_entrypoint(chunk_ukey, compilation).map_or(chunk_ukey, |group| {
      group.get_runtime_chunk(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
    })
  }

  pub(crate) fn get_entry_chunk(chunk_ukey: ChunkUkey, compilation: &Compilation) -> ChunkUkey {
    Self::get_entrypoint(chunk_ukey, compilation)
      .map_or(chunk_ukey, ChunkGroup::get_entrypoint_chunk)
  }

  pub(crate) async fn render_chunk(
    &self,
    compilation: &Compilation,
    chunk_ukey: &ChunkUkey,
    asset_info: &mut AssetInfo,
    runtime_template: &RuntimeCodeTemplate,
  ) -> Result<Option<RenderSource>> {
    let module_graph = compilation.get_module_graph();

    // In this phase we only read from the lock, no write happen in this phase, the
    // next write happen only happen for next compile start
    let chunk_link_guard = self.links.borrow();
    let chunk_link = &chunk_link_guard[chunk_ukey];

    let mut chunk_init_fragments: Vec<Box<dyn InitFragment<ChunkRenderContext> + 'static>> =
      chunk_link.init_fragments.clone();

    let mut replace_auto_public_path = false;
    let mut replace_static_url = false;

    // Same as above, we can only read here, the write happen only at the finish_modules phase
    let concatenated_modules_map = self.concatenated_modules_map.read().await;

    let chunk = get_chunk(compilation, *chunk_ukey);
    let runtime_chunk_ukey = Self::get_runtime_chunk(*chunk_ukey, compilation);
    let entry_chunk_ukey = Self::get_entry_chunk(*chunk_ukey, compilation);
    let is_separate_runtime_chunk =
      runtime_chunk_ukey == *chunk_ukey && runtime_chunk_ukey != entry_chunk_ukey;
    let module_runtime_template = runtime_template;
    let runtime_mode_renderer = renderer_for(runtime_template.render_mode());
    let filename_template = get_js_chunk_filename_template(
      chunk,
      &compilation.options.output,
      &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
    );

    asset_info.set_javascript_module(true);

    let output_path = compilation
      .get_path_with_info(
        &filename_template,
        PathData::default()
          .chunk(chunk.ukey(), compilation)
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
        asset_info,
      )
      .await?;

    let mut runtime_requirements =
      *ChunkGraph::get_chunk_runtime_requirements(compilation, chunk_ukey);

    // Wrapped modules are emitted as direct, chunk-local initializers. The
    // shared helper owns each module's cache in the initializer closure.
    let mut decl_source = ConcatSource::default();

    if chunk_link.wrapped_runtime.esm.is_some() {
      decl_source.add(render_esm_wrapper(chunk_link));
    }

    let lazy_only_required = chunk_link
      .required
      .iter()
      .filter_map(|(target, interop)| {
        let mut has_lazy_source = false;
        let mut has_eager_source = interop.from_module.is_empty();
        for source in &interop.from_module {
          match concatenated_modules_map.get(source) {
            Some(rspack_core::ModuleInfo::Concatenated(info)) if info.initializer.is_some() => {
              has_lazy_source = true;
            }
            Some(rspack_core::ModuleInfo::Concatenated(_)) => has_eager_source = true,
            Some(rspack_core::ModuleInfo::Wrapped(_)) | None => {}
            Some(rspack_core::ModuleInfo::External(_)) => {
              unreachable!("external module info is not used by modern-module rendering")
            }
          }
        }
        (has_lazy_source && !has_eager_source).then_some(*target)
      })
      .collect::<IdentifierIndexSet>();
    let mut lazy_required_declarations = FxIndexSet::default();
    for target in &lazy_only_required {
      lazy_required_declarations.extend(chunk_link.required[target].declaration_names().cloned());
    }
    if !lazy_required_declarations.is_empty() {
      decl_source.add(RawStringSource::from(format!(
        "var {};\n",
        lazy_required_declarations
          .iter()
          .map(Atom::as_str)
          .collect::<Vec<_>>()
          .join(", ")
      )));
    }

    if !chunk_link.wrapped_modules.is_empty() {
      let hooks = JsPlugin::get_compilation_hooks(compilation.id());
      let cjs_plan = chunk_link
        .wrapped_runtime
        .cjs
        .as_ref()
        .expect("chunk with wrapped modules should have a CJS wrapper plan");
      decl_source.add(render_cjs_wrapper(cjs_plan));
      for m in chunk_link.wrapped_modules.iter() {
        let module = module_graph
          .module_by_identifier(m)
          .expect("should have module");

        let hooks = hooks.read().await;
        let Some((module_source, init_frags, init_frags2)) = render_module(
          compilation,
          chunk_ukey,
          module.as_ref(),
          true,
          false,
          &output_path,
          &hooks,
          module_runtime_template,
        )
        .await?
        else {
          continue;
        };
        drop(hooks);

        chunk_init_fragments.extend(init_frags);
        chunk_init_fragments.extend(init_frags2);
        let codegen_result = compilation.code_generation_results.get(m, None);
        if codegen_result
          .data
          .get::<CodeGenerationModuleReferences>()
          .is_some_and(CodeGenerationModuleReferences::needs_static_url_replacement)
        {
          replace_static_url = true;
        }
        let module_source = relocate_module_references(
          module_source.clone(),
          codegen_result.data.get::<CodeGenerationModuleReferences>(),
          compilation,
          *chunk_ukey,
          chunk_link,
          &chunk_link_guard,
          &concatenated_modules_map,
        )?;
        let info = concatenated_modules_map
          .get(m)
          .expect("wrapped module should have link info")
          .as_wrapped();
        let initializer = info
          .initializer_name
          .as_ref()
          .expect("wrapped module should have an initializer name");
        let module_argument =
          module_runtime_template.render_module_argument(module.get_module_argument());
        let exports_argument =
          module_runtime_template.render_exports_argument(module.get_exports_argument());
        let helper = &cjs_plan.helper;

        decl_source.add(RawStringSource::from(format!(
          "var {initializer} = /*#__PURE__*/ {helper}(function({module_argument}, {exports_argument}) {{\n"
        )));
        decl_source.add(module_source);
        decl_source.add(RawStringSource::from_static("\n});\n\n"));
      }
    }

    // present as
    // a.js -> (imported symbol, local symbol)
    // we use rspack_require to load modules that are not scope hoisted
    // and we should also deconflict them
    // const symbol = __rspack_require('./main.js')

    // render cross module links
    let mut runtime_source = ConcatSource::default();
    let mut import_source = ConcatSource::default();
    let mut render_source = ConcatSource::default();
    let mut export_specifiers: FxIndexSet<Cow<str>> = Default::default();
    let mut export_default = None;
    let mut has_default_export = false;
    let mut imported_chunks = FxIndexMap::<ChunkUkey, FxHashMap<Atom, Atom>>::default();
    // render webpack runtime
    if chunk.has_runtime(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey) {
      if is_separate_runtime_chunk {
        asset_info
          .extras
          .insert(RSPACK_ESM_RUNTIME_CHUNK.into(), "true".into());
      }
      // render chunk needs to render *all* runtimes in the whole tree
      let tree_runtime_requirements = without_module_loader_runtime_globals(
        *ChunkGraph::get_tree_runtime_requirements(compilation, chunk_ukey),
      );
      // A pure runtime chunk has no entry modules of its own; it was split off
      // by optimize_runtime_chunks and only exists to export __rspack_require.
      // An entry-with-runtime chunk (runtimeChunk: false, not split) uses
      // __rspack_require internally but must not export it.
      let is_pure_runtime_chunk = compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_chunk_entry_modules(chunk_ukey)
        .is_empty();

      // When the entry chunk IS the runtime chunk (runtimeChunk: false without split)
      // and no runtime modules actually use the __rspack_require scope, strip
      // REQUIRE_SCOPE so we don't emit a useless `var __rspack_require = {};`.
      let has_runtime_modules = compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_chunk_runtime_modules_iterable(chunk_ukey)
        .next()
        .is_some();
      let mut effective_tree_requirements = if !is_pure_runtime_chunk
        && !has_runtime_modules
        && !tree_runtime_requirements.contains(RuntimeGlobals::REQUIRE)
      {
        tree_runtime_requirements.difference(RuntimeGlobals::REQUIRE_SCOPE)
      } else {
        tree_runtime_requirements
      };
      let tree_uses_explicit_runtime_scope = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .keys()
        .any(|tree_chunk| {
          Self::get_runtime_chunk(*tree_chunk, compilation) == *chunk_ukey
            && chunk_uses_explicit_runtime_scope(compilation, tree_chunk)
        });
      if tree_uses_explicit_runtime_scope {
        effective_tree_requirements.insert(RuntimeGlobals::REQUIRE | RuntimeGlobals::REQUIRE_SCOPE);
      }
      let exports_require_via_runtime_module = compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_chunk_runtime_modules_iterable(chunk_ukey)
        .any(|runtime_module_id| {
          compilation
            .runtime_modules
            .get(runtime_module_id)
            .is_some_and(|module| module.get_constructor_name() == "ExportRequireRuntimeModule")
        });

      let runtime_scope_imported_by_child = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .keys()
        .any(|other_chunk| {
          other_chunk != chunk_ukey
            && Self::get_runtime_chunk(*other_chunk, compilation) == *chunk_ukey
            && (without_module_loader_runtime_globals(*ChunkGraph::get_chunk_runtime_requirements(
              compilation,
              other_chunk,
            ))
            .contains(RuntimeGlobals::REQUIRE_SCOPE)
              || chunk_uses_explicit_runtime_scope(compilation, other_chunk))
        });
      let should_export_require_from_runtime =
        !exports_require_via_runtime_module && runtime_scope_imported_by_child;
      let runtimes = runtime_mode_renderer.render_runtime(RuntimeRenderContext {
        chunk_ukey,
        compilation,
        runtime_requirements: effective_tree_requirements,
        runtime_template: module_runtime_template,
        should_export_require: should_export_require_from_runtime,
      });

      runtime_source.add(runtimes);
      runtime_source.add(RawStringSource::from_static("\n"));
      runtime_source
        .add(render_runtime_modules(compilation, chunk_ukey, module_runtime_template).await?);
      runtime_source.add(RawStringSource::from_static("\n"));

      // Native ESM child chunks import the runtime scope as an ordinary ESM
      // binding. Export it directly from any runtime-bearing chunk; this also
      // covers runtimeChunk: false, where there is no registry-based
      // ExportRequireRuntimeModule anymore.
      if let Some(runtime_export) = runtime_mode_renderer
        .render_direct_runtime_export(runtime_template, should_export_require_from_runtime)
      {
        export_specifiers.insert(Cow::Owned(runtime_export));
      }
    }

    // render namespace object before render module contents
    let mut namespace_object_sources = chunk_link
      .namespace_object_sources
      .iter()
      .collect::<Vec<_>>();
    namespace_object_sources.sort_by_key(|(a, _)| *a);
    for (_, namespace) in namespace_object_sources {
      render_source.add(RawStringSource::from(format!("{namespace}\n")));
    }

    let mut already_required = IdentifierIndexSet::default();

    for m in &chunk_link.hoisted_modules {
      let info = concatenated_modules_map
        .get(m)
        .expect("should have info")
        .as_concatenated();
      if info.public_path_auto_replacement == Some(true) {
        replace_auto_public_path = true;
      }
      if info.static_url_replacement {
        replace_static_url = true;
      }
      let source = Self::render_module(info, chunk_link)?;
      let wrapped_required = Self::render_wrapped_required(
        *m,
        compilation,
        chunk_link,
        &mut already_required,
        &lazy_only_required,
        module_runtime_template,
      );
      let mut dependency_initializers = render_static_initializer_dependencies(
        *m,
        compilation,
        chunk_link,
        &concatenated_modules_map,
      );
      let mut module_source = ConcatSource::default();
      if info.initializer.is_some() {
        dependency_initializers.push_str(&wrapped_required.source().into_string_lossy());
        module_source.add(render_initializer(
          info,
          source,
          &chunk_link
            .wrapped_runtime
            .esm
            .as_ref()
            .expect("initializer-backed module should have an ESM wrapper")
            .helper,
          dependency_initializers,
          rspack_core::ModuleGraph::is_async(&compilation.async_modules_artifact, m),
        ));
      } else {
        module_source.add(wrapped_required);
        module_source.add(RawStringSource::from(dependency_initializers));
        module_source.add(source);
      }
      let codegen_result = compilation.code_generation_results.get(m, None);
      if codegen_result
        .data
        .get::<CodeGenerationModuleReferences>()
        .is_some_and(CodeGenerationModuleReferences::needs_static_url_replacement)
      {
        replace_static_url = true;
      }
      let module_source = relocate_module_references(
        module_source.boxed(),
        codegen_result.data.get::<CodeGenerationModuleReferences>(),
        compilation,
        *chunk_ukey,
        chunk_link,
        &chunk_link_guard,
        &concatenated_modules_map,
      )?;

      if !matches!(compilation.options.output.pathinfo, PathInfo::Bool(false)) {
        render_source.add(RawStringSource::from(format!(
          "// {}\n",
          ChunkGraph::get_module_id(&compilation.module_ids_artifact, *m).map_or_else(
            || {
              let module = module_graph
                .module_by_identifier(m)
                .expect("should have module");
              module
                .readable_identifier(&compilation.options.context)
                .to_string()
            },
            |id| { id.to_string() },
          )
        )));
      }

      render_source.add(module_source);
      render_source.add(RawStringSource::from_static("\n"));

      chunk_init_fragments.extend(info.chunk_init_fragments.clone());

      if info.interop_namespace_object_used {
        render_source.add(RawStringSource::from(format!(
          "var {} = /*#__PURE__*/{}({}, 2);\n",
          info
            .interop_namespace_object_name
            .clone()
            .expect("should have interop_namespace_object_name"),
          module_runtime_template
            .render_runtime_globals(&RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT),
          info
            .namespace_object_name
            .as_ref()
            .expect("should have name")
        )));
      }

      if info.interop_namespace_object2_used {
        render_source.add(RawStringSource::from(format!(
          "var {} = /*#__PURE__*/{}({});\n",
          info
            .interop_namespace_object2_name
            .clone()
            .expect("should have interop_namespace_object2_name"),
          module_runtime_template
            .render_runtime_globals(&RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT),
          info
            .namespace_object_name
            .as_ref()
            .expect("should have name")
        )));
      }

      if info.interop_default_access_used {
        render_source.add(RawStringSource::from(format!(
          "\nvar {} = /*#__PURE__*/{}({});",
          info
            .interop_default_access_name
            .clone()
            .expect("should have interop_default_access_name"),
          module_runtime_template
            .render_runtime_globals(&RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT),
          info
            .namespace_object_name
            .as_ref()
            .expect("should have name")
        )));
      }
    }

    for entry_module in compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_entry_modules(chunk_ukey)
    {
      if !matches!(
        concatenated_modules_map.get(&entry_module),
        Some(rspack_core::ModuleInfo::Concatenated(info)) if info.initializer.is_some()
      ) {
        continue;
      }
      let Some(initializer) = chunk_link.module_initializers.get(&entry_module) else {
        continue;
      };
      render_source.add(RawStringSource::from(format!(
        "{}{initializer}();\n",
        if rspack_core::ModuleGraph::is_async(&compilation.async_modules_artifact, &entry_module,) {
          "await "
        } else {
          ""
        }
      )));
    }

    for (m, required_info) in &chunk_link.required {
      // Skip CSS-only modules (native CSS or extract-css CssModule). They
      // are loaded by the CSS plugin runtime, not by `__rspack_require`.
      if let Some(module) = module_graph.module_by_identifier(m)
        && is_css_only_module(module.as_ref(), module_graph)
      {
        continue;
      }
      if lazy_only_required.contains(m) {
        already_required.insert(*m);
        continue;
      }
      if already_required.insert(*m) {
        render_source.add(required_info.render(compilation, module_runtime_template));
        render_source.add(RawStringSource::from_static("\n"));
      }
    }

    // render imports and exports to other chunks
    for required_module in already_required {
      let target_chunk = Self::get_module_chunk(required_module, compilation)?;
      if &target_chunk != chunk_ukey {
        // Skip chunks that have no JavaScript modules. CSS-only chunks
        // produced by `preserveModules` are loaded by the CSS plugin
        // runtime, not by importing the chunk's JS file (it has none).
        if !compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .has_chunk_module_by_source_type(&target_chunk, SourceType::JavaScript, module_graph)
        {
          continue;
        }
        imported_chunks.entry(target_chunk).or_default();
      }
    }

    runtime_requirements = without_module_loader_runtime_globals(runtime_requirements);
    if chunk_uses_explicit_runtime_scope(compilation, chunk_ukey) {
      runtime_requirements.insert(RuntimeGlobals::REQUIRE | RuntimeGlobals::REQUIRE_SCOPE);
    }

    import_source.add(
      runtime_mode_renderer.render_runtime_imports(RuntimeImportRenderContext {
        compilation,
        chunk_ukey,
        runtime_chunk_ukey: &runtime_chunk_ukey,
        chunk_link,
        runtime_requirements,
        runtime_template: module_runtime_template,
      }),
    );

    for (id, imports) in &chunk_link.imports {
      let chunk = Self::get_module_chunk(*id, compilation)?;
      if &chunk == chunk_ukey {
        // ignore self import
        continue;
      }

      // Skip chunks that have no JavaScript modules (e.g. CSS-only chunks
      // produced by `preserveModules` for native CSS or extract-css). The
      // CSS chunk is handled by the CSS plugin's own runtime, not via a
      // JS-side bare import.
      if !compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .has_chunk_module_by_source_type(&chunk, SourceType::JavaScript, module_graph)
      {
        continue;
      }

      let imported_symbols = imported_chunks.entry(chunk).or_default();
      if imports.is_empty() {
        continue;
      }

      for (imported, local) in imports {
        imported_symbols.insert(imported.clone(), local.clone());
      }
    }

    for (chunk, imported) in &imported_chunks {
      if imported.is_empty()
        && chunk_link
          .re_exports()
          .contains_key(&ReExportFrom::Chunk(*chunk))
      {
        continue;
      }
      let chunk = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .expect_get(chunk);

      if imported.is_empty() {
        import_source.add(RawStringSource::from(format!(
          "import \"__RSPACK_ESM_CHUNK_{}\";\n",
          chunk.expect_id().as_str()
        )));
      } else {
        let mut stmt = String::with_capacity(imported.len() * 30 + 40);
        stmt.push_str("import { ");
        for (i, (imported_sym, local)) in imported.iter().enumerate() {
          if i > 0 {
            stmt.push_str(", ");
          }
          let imported_name = export_name(imported_sym).expect("should have export_name");
          if imported_sym == local {
            stmt.push_str(&imported_name);
          } else {
            let local_name = export_name(local).expect("should have export_name");
            stmt.push_str(&imported_name);
            stmt.push_str(" as ");
            stmt.push_str(&local_name);
          }
        }
        stmt.push_str(" } from \"__RSPACK_ESM_CHUNK_");
        stmt.push_str(chunk.expect_id().as_str());
        stmt.push_str("\";\n");
        import_source.add(RawStringSource::from(stmt));
      }
    }

    if !imported_chunks.is_empty() || !chunk_link.raw_import_stmts.is_empty() {
      import_source.add(RawStringSource::from_static("\n"));
    }

    // render init fragments
    let mut final_source = ConcatSource::default();
    if let Some(hashbang) = &chunk_link.hashbang {
      final_source.add(RawStringSource::from(hashbang.clone()));
    }
    for directive in &chunk_link.directives {
      final_source.add(RawStringSource::from(format!("{directive}\n")));
    }
    final_source.add(import_source.boxed());
    final_source.add(render_init_fragments(
      ConcatSource::new([
        runtime_source.boxed(),
        decl_source.boxed(),
        render_source.boxed(),
      ])
      .boxed(),
      chunk_init_fragments,
      &mut ChunkRenderContext {},
    )?);

    let mut exports = chunk_link.exports().iter().collect::<Vec<_>>();
    exports.sort_by(|a, b| a.0.cmp(b.0));
    for decl_before_export in chunk_link.decl_before_exports.iter() {
      final_source.add(RawStringSource::from(decl_before_export.clone()));
    }

    for (raw_symbol, exports) in exports {
      let mut exports = exports.iter().collect::<Vec<_>>();
      exports.sort_unstable();
      for exported_name in exports {
        let is_default = exported_name.as_str() == "default";

        if is_default {
          if !has_default_export {
            has_default_export = true;
            if let Ok(raw_symbol_name) = export_name(raw_symbol) {
              // `export default binding` snapshots the binding's current value.
              // An initializer-backed module assigns it later, so use an export
              // specifier to retain ESM live-binding semantics.
              export_specifiers.insert(Cow::Owned(format!("{raw_symbol_name} as default")));
            } else {
              export_default = Some(raw_symbol);
            }
          } else {
            // multiple export default
            export_specifiers.insert(Cow::Owned(
              export_name(raw_symbol)
                .expect("should have export_name")
                .into_owned(),
            ));
          }
        } else if raw_symbol == exported_name {
          export_specifiers.insert(Cow::Owned(
            export_name(raw_symbol)
              .expect("should have export_name")
              .into_owned(),
          ));
        } else {
          let raw_symbol_name = export_name(raw_symbol).expect("should have export_name");
          let exported_name_str = export_name(exported_name).expect("should have export_name");
          export_specifiers.insert(Cow::Owned(format!(
            "{raw_symbol_name} as {exported_name_str}"
          )));
        }
      }
    }

    // Keep side-effect-only Node chunks explicitly in ESM form.
    // We only emit `export {};` when the chunk would otherwise render no export syntax at all.
    let should_render_empty_export = compilation.platform.is_node()
      && !runtime_mode_renderer.renders_inline_runtime_exports(compilation, chunk_ukey)
      && export_specifiers.is_empty()
      && chunk_link.raw_star_exports.is_empty()
      && chunk_link.re_exports().is_empty()
      && !has_default_export;

    if !export_specifiers.is_empty() {
      let mut export_str = String::with_capacity(export_specifiers.len() * 20);
      export_str.push_str("export { ");
      for (i, s) in export_specifiers.iter().enumerate() {
        if i > 0 {
          export_str.push_str(", ");
        }
        export_str.push_str(s);
      }
      export_str.push_str(" };\n");
      final_source.add(RawStringSource::from(export_str));
    }

    // render star exports
    for (source, export_names) in &chunk_link.raw_star_exports {
      for name in export_names {
        if name == "*" {
          final_source.add(RawStringSource::from(format!(
            "export * from {};\n",
            rspack_util::json_stringify_str(source)
          )));
        } else {
          let name_str = export_name(name).expect("should have export_name");
          final_source.add(RawStringSource::from(format!(
            "export * as {name_str} from {};\n",
            rspack_util::json_stringify_str(source)
          )));
        }
      }
    }

    // render re-exports
    for (re_export_from, export_symbols) in chunk_link.re_exports() {
      let mut export_symbols = export_symbols.iter().collect::<Vec<_>>();
      export_symbols.sort_by(|a, b| a.0.cmp(b.0));

      let from_str = match re_export_from {
        crate::chunk_link::ReExportFrom::Chunk(chunk_ukey) => {
          let chunk = compilation
            .build_chunk_graph_artifact
            .chunk_by_ukey
            .expect_get(chunk_ukey);
          Cow::Owned(format!("__RSPACK_ESM_CHUNK_{}", chunk.expect_id().as_str()))
        }
        crate::chunk_link::ReExportFrom::Request(request) => Cow::Borrowed(request.as_str()),
      };
      let mut stmt = String::with_capacity(export_symbols.len() * 30 + from_str.len() + 30);
      stmt.push_str("export { ");
      let mut first = true;
      for (imported, exports) in &export_symbols {
        let mut sorted_exports = exports.iter().collect::<Vec<_>>();
        sorted_exports.sort_unstable();
        let imported_name = export_name(imported).expect("should have export_name");
        for exported_name in sorted_exports {
          if !first {
            stmt.push_str(", ");
          }
          first = false;
          stmt.push_str(&imported_name);
          if *imported != exported_name {
            let exported_name_str = export_name(exported_name).expect("should have export_name");
            stmt.push_str(" as ");
            stmt.push_str(&exported_name_str);
          }
        }
      }
      stmt.push_str(" } from \"");
      stmt.push_str(&from_str);
      stmt.push_str("\";\n");
      final_source.add(RawStringSource::from(stmt));
    }

    if let Some(default_export) = export_default {
      final_source.add(RawStringSource::from(format!(
        "export default {default_export};\n",
      )));
    }

    if should_render_empty_export {
      final_source.add(RawStringSource::from_static("export {};\n"));
    }

    let final_source = if replace_auto_public_path {
      let mut replace_source = ReplaceSource::new(final_source);
      let mut replacement = vec![];
      for (start, matched) in replace_source
        .source()
        .into_string_lossy()
        .match_indices(AUTO_PUBLIC_PATH_PLACEHOLDER)
      {
        let start = start as u32;
        let end = (start as usize + matched.len()) as u32;
        let relative = get_undo_path(
          &output_path,
          compilation.options.output.path.to_string(),
          true,
        );
        replacement.push((start, end, relative));
      }

      for (start, end, relative) in replacement {
        replace_source.replace(start, end, relative, None);
      }

      // concate module does this by render_module()
      // however esm module does not have concate module,
      // some replacement needs to be done here
      replace_source.boxed()
    } else {
      Arc::new(final_source)
    };

    let final_source = if replace_static_url {
      // concate module does this by render_module()
      // however esm module does not have concate module,
      // some replacement needs to be done here
      replace_static_url_placeholders(compilation, None, &output_path, final_source).await?
    } else {
      final_source
    };
    Ok(Some(RenderSource {
      source: final_source,
    }))
  }

  pub fn render_module(
    info: &ConcatenatedModuleInfo,
    chunk_link: &ChunkLinkContext,
  ) -> Result<ReplaceSource> {
    let Some(mut source) = info.source.clone() else {
      return Err(rspack_error::Error::error(format!(
        "module: {} has no source",
        info.module
      )));
    };

    for ((atom, ctxt), refs) in &info.binding_to_ref {
      if ctxt == &info.global_ctxt
        && let Some(binding_ref) = chunk_link.refs.get(atom.as_str())
      {
        let final_name = match binding_ref {
          Ref::Symbol(symbol_ref) => Cow::Owned(symbol_ref.render()),
          Ref::Inline(inline) => Cow::Borrowed(inline),
        };

        for ident in refs {
          let name = if ident.shorthand {
            Cow::Owned(format!("{}: {}", &ident.id.sym, &final_name))
          } else {
            final_name.clone()
          };
          source.replace(
            ident.id.span.real_lo(),
            ident.id.span.real_hi() + 2,
            name.into_owned(),
            None,
          );
        }
      }
    }

    for ident in &info.idents {
      if ident.id.ctxt != info.module_ctxt {
        continue;
      }

      if let Some(internal_name) = info.get_internal_name(&ident.id.sym) {
        let name = if ident.shorthand {
          format!("{}: {}", &ident.id.sym, &internal_name)
        } else {
          internal_name.to_string()
        };
        source.replace(ident.id.span.real_lo(), ident.id.span.real_hi(), name, None);
      }
    }

    Ok(source)
  }

  pub fn render_wrapped_required(
    root: ModuleIdentifier,
    compilation: &Compilation,
    chunk_link: &ChunkLinkContext,
    already_required: &mut IdentifierIndexSet,
    lazy_only_required: &IdentifierIndexSet,
    runtime_template: &RuntimeCodeTemplate,
  ) -> ConcatSource {
    let mut source = ConcatSource::default();
    let module_graph = compilation.get_module_graph();

    for (id, interop_info) in &chunk_link.required {
      if !interop_info.from_module.contains(&root) {
        continue;
      }
      // CSS-only modules are loaded by the CSS runtime and have no wrapped
      // JavaScript initializer to render here.
      if let Some(module) = module_graph.module_by_identifier(id)
        && is_css_only_module(module.as_ref(), module_graph)
      {
        continue;
      }
      if lazy_only_required.contains(id) {
        source.add(interop_info.render_assignments(compilation, runtime_template));
        already_required.insert(*id);
        continue;
      }
      if !already_required.insert(*id) {
        continue;
      }

      source.add(interop_info.render(compilation, runtime_template));
    }

    source
  }
}
