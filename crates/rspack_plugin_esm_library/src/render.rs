use std::{borrow::Cow, sync::Arc};

use rspack_collections::{IdentifierIndexSet, UkeyIndexMap, UkeySet};
use rspack_core::{
  AssetInfo, Chunk, ChunkGraph, ChunkRenderContext, ChunkUkey, CodeGenerationDataFilename,
  Compilation, ConcatenatedModuleInfo, DependencyId, InitFragment, ModuleIdentifier, PathData,
  PathInfo, RuntimeCodeTemplate, RuntimeGlobals, RuntimeVariable, SourceType, export_name,
  get_js_chunk_filename_template, get_undo_path, render_imports, render_init_fragments,
  rspack_sources::{ConcatSource, RawStringSource, ReplaceSource, Source, SourceExt},
};
use rspack_error::Result;
use rspack_plugin_javascript::{
  JsPlugin, RenderSource,
  dependency::{URL_STATIC_PLACEHOLDER, URL_STATIC_PLACEHOLDER_RE},
  runtime::{AUTO_PUBLIC_PATH_PLACEHOLDER, render_module, render_runtime_modules},
};
use rspack_plugin_runtime::EXPORT_REQUIRE_RUNTIME_MODULE_ID;
use rspack_util::{
  SpanExt,
  atom::Atom,
  fx_hash::{FxHashMap, FxIndexSet},
};

use crate::{
  chunk_link::{ChunkLinkContext, ReExportFrom, Ref},
  plugin::RSPACK_ESM_RUNTIME_CHUNK,
  runtime::EsmRegisterModuleRuntimeModule,
};

#[inline]
fn get_chunk(compilation: &Compilation, chunk_ukey: ChunkUkey) -> &Chunk {
  compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(&chunk_ukey)
}

use crate::EsmLibraryPlugin;

impl EsmLibraryPlugin {
  pub(crate) fn get_runtime_chunk(chunk_ukey: ChunkUkey, compilation: &Compilation) -> ChunkUkey {
    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(&chunk_ukey);
    let group = chunk
      .groups()
      .iter()
      .next()
      .expect("should have at least one group");
    let group = compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .expect_get(group);
    let mut stack = vec![group];
    let mut visited = UkeySet::default();

    while let Some(group) = stack.pop() {
      if !visited.insert(group.ukey) {
        continue;
      }

      if group.kind.is_entrypoint() {
        return group
          .get_runtime_chunk(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey);
      }

      stack.extend(group.parents_iterable().map(|group| {
        compilation
          .build_chunk_graph_artifact
          .chunk_group_by_ukey
          .expect_get(group)
      }));
    }

    unreachable!("chunk should have at least one ancestor that is entrypoint")
  }

  pub(crate) async fn render_chunk(
    &self,
    compilation: &Compilation,
    chunk_ukey: &ChunkUkey,
    asset_info: &mut AssetInfo,
    runtime_template: &RuntimeCodeTemplate<'_>,
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

    // modules that are not scope hoisted, store in runtime
    let mut decl_source = ConcatSource::default();

    if !chunk_link.decl_modules.is_empty() {
      let hooks = JsPlugin::get_compilation_hooks(compilation.id());

      let mut decl_inner = ConcatSource::default();
      for m in chunk_link.decl_modules.iter() {
        let module = module_graph
          .module_by_identifier(m)
          .expect("should have module");

        let hooks = hooks.read().await;
        let Some((module_source, init_frags, init_frags2)) = render_module(
          compilation,
          chunk_ukey,
          module.as_ref(),
          true,
          true,
          &output_path,
          &hooks,
          runtime_template,
        )
        .await?
        else {
          continue;
        };
        drop(hooks);

        chunk_init_fragments.extend(init_frags);
        chunk_init_fragments.extend(init_frags2);
        decl_inner.add(module_source.clone());
      }

      if !decl_inner.source().is_empty() {
        // __webpack_require__.add({ "./src/main.js"(require, exports) { ... } })
        decl_source.add(RawStringSource::from(format!(
          "{}({{\n",
          EsmRegisterModuleRuntimeModule::runtime_id(
            &compilation.runtime_template.create_runtime_code_template()
          )
        )));
        decl_source.add(decl_inner);
        decl_source.add(RawStringSource::from_static("});\n"));
      }
    }

    // present as
    // a.js -> (imported symbol, local symbol)
    // we use webpack_require to load modules that are not scope hoisted
    // and we should also deconflict them
    // const symbol = __webpack_require__('./main.js')

    // render cross module links
    let mut runtime_source = ConcatSource::default();
    let mut import_source = ConcatSource::default();
    let mut render_source = ConcatSource::default();
    let mut export_specifiers: FxIndexSet<Cow<str>> = Default::default();
    let mut export_default = None;
    let mut imported_chunks = UkeyIndexMap::<ChunkUkey, FxHashMap<Atom, Atom>>::default();
    let mut runtime_requirements =
      *ChunkGraph::get_chunk_runtime_requirements(compilation, chunk_ukey);

    // render webpack runtime
    if chunk.has_runtime(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey) {
      asset_info
        .extras
        .insert(RSPACK_ESM_RUNTIME_CHUNK.into(), "true".into());
      // render chunk needs to render *all* runtimes in the whole tree
      let tree_runtime_requirements =
        ChunkGraph::get_tree_runtime_requirements(compilation, chunk_ukey);
      if tree_runtime_requirements.contains(RuntimeGlobals::MODULE_FACTORIES) {
        runtime_source.add(RawStringSource::from(format!(
          r#"
var {} = {{}};
"#,
          runtime_template.render_runtime_variable(&RuntimeVariable::Modules)
        )));
      }
      let runtimes = Self::render_runtime(
        chunk_ukey,
        compilation,
        *tree_runtime_requirements,
        runtime_template,
      )
      .await?;

      runtime_source.add(runtimes);
      runtime_source.add(RawStringSource::from_static("\n"));
      runtime_source.add(render_runtime_modules(compilation, chunk_ukey, runtime_template).await?);
      runtime_source.add(RawStringSource::from_static("\n"));

      // EXPORT_WEBPACK_REQUIRE_RUNTIME_MODULE runtime will export __webpack_require__ already
      if !compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_chunk_runtime_modules_iterable(chunk_ukey)
        .any(|m| m.contains(EXPORT_REQUIRE_RUNTIME_MODULE_ID))
        && tree_runtime_requirements.contains(RuntimeGlobals::REQUIRE)
      {
        export_specifiers.insert(Cow::Owned(
          runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
        ));
      }
    }

    // render namespace object before render module contents
    let mut namespace_object_sources = chunk_link
      .namespace_object_sources
      .iter()
      .collect::<Vec<_>>();
    namespace_object_sources.sort_by(|(a, _), (b, _)| a.cmp(b));
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

      render_source.add(Self::render_external_required(
        *m,
        compilation,
        chunk_link,
        &mut already_required,
        runtime_template,
      ));
      render_source.add(source);
      render_source.add(RawStringSource::from_static("\n"));

      chunk_init_fragments.extend(info.chunk_init_fragments.clone());

      if info.interop_namespace_object_used {
        render_source.add(RawStringSource::from(format!(
          "var {} = /*#__PURE__*/{}({}, 2);\n",
          info
            .interop_namespace_object_name
            .clone()
            .expect("should have interop_namespace_object_name"),
          runtime_template.render_runtime_globals(&RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT),
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
          runtime_template.render_runtime_globals(&RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT),
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
          runtime_template.render_runtime_globals(&RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT),
          info
            .namespace_object_name
            .as_ref()
            .expect("should have name")
        )));
      }
    }

    for (m, required_info) in &chunk_link.required {
      if already_required.insert(*m) {
        runtime_requirements.insert(RuntimeGlobals::REQUIRE);
        render_source.add(required_info.render(compilation, runtime_template));
        render_source.add(RawStringSource::from_static("\n"));
      }
    }

    // render imports and exports to other chunks
    for required_module in already_required {
      runtime_requirements.insert(RuntimeGlobals::REQUIRE);
      let target_chunk = Self::get_module_chunk(required_module, compilation);
      if &target_chunk != chunk_ukey {
        imported_chunks.entry(target_chunk).or_default();
      }
    }

    if !runtime_requirements.is_empty() {
      let runtime_chunk = Self::get_runtime_chunk(*chunk_ukey, compilation);
      if &runtime_chunk != chunk_ukey && runtime_requirements.contains(RuntimeGlobals::REQUIRE) {
        let runtime_chunk = compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get(&runtime_chunk);

        import_source.add(RawStringSource::from(format!(
          "import {{ {} }} from \"__RSPACK_ESM_CHUNK_{}\";\n",
          runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
          runtime_chunk.expect_id().as_str()
        )));
      }
    }

    for ((source, attr), import_spec) in &chunk_link.raw_import_stmts {
      import_source.add(RawStringSource::from(render_imports(
        source,
        attr.as_deref(),
        import_spec,
      )));
    }

    for (id, imports) in &chunk_link.imports {
      let chunk = Self::get_module_chunk(*id, compilation);
      if &chunk == chunk_ukey {
        // ignore self import
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

      import_source.add(RawStringSource::from(format!(
        "import {}\"__RSPACK_ESM_CHUNK_{}\";\n",
        if imported.is_empty() {
          String::new()
        } else {
          format!(
            "{{ {} }} from ",
            imported
              .iter()
              .map(|(imported, local)| {
                let imported_name = export_name(imported).expect("should have export_name");
                if imported == local {
                  imported_name.into_owned()
                } else {
                  let local_name = export_name(local).expect("should have export_name");
                  format!("{imported_name} as {local_name}")
                }
              })
              .collect::<Vec<_>>()
              .join(", ")
          )
        },
        chunk.expect_id().as_str()
      )));
    }

    if !imported_chunks.is_empty() || !chunk_link.raw_import_stmts.is_empty() {
      import_source.add(RawStringSource::from_static("\n"));
    }

    // render init fragments
    let mut final_source = ConcatSource::new([
      render_init_fragments(
        import_source.boxed(),
        chunk_init_fragments,
        &mut ChunkRenderContext {},
      )?,
      ConcatSource::new([
        runtime_source.boxed(),
        decl_source.boxed(),
        render_source.boxed(),
      ])
      .boxed(),
    ]);

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
          if export_default.is_none() {
            export_default = Some(raw_symbol);
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

    if !export_specifiers.is_empty() {
      final_source.add(RawStringSource::from(format!(
        "export {{ {} }};\n",
        export_specifiers
          .into_iter()
          .map(|s| s.to_string())
          .collect::<Vec<_>>()
          .join(", ")
      )));
    }

    // render star exports
    for (source, export_names) in &chunk_link.raw_star_exports {
      for name in export_names {
        if name == "*" {
          final_source.add(RawStringSource::from(format!(
            "export * from {};\n",
            serde_json::to_string(source).expect("should have correct request")
          )));
        } else {
          let name_str = export_name(name).expect("should have export_name");
          final_source.add(RawStringSource::from(format!(
            "export * as {name_str} from {};\n",
            serde_json::to_string(source).expect("should have correct request")
          )));
        }
      }
    }

    // render re-exports
    for (re_export_from, export_symbols) in chunk_link.re_exports() {
      let mut export_symbols = export_symbols.iter().collect::<Vec<_>>();
      export_symbols.sort_by(|a, b| a.0.cmp(b.0));

      final_source.add(RawStringSource::from(format!(
        "export {{ {} }} from \"{}\";\n",
        export_symbols
          .iter()
          .flat_map(|(imported, exports)| {
            let mut vec = exports.iter().collect::<Vec<_>>();
            vec.sort_unstable();
            let imported_name = export_name(imported)
              .expect("should have export_name")
              .into_owned();
            vec.into_iter().map(move |exported_name| {
              if *imported == exported_name {
                imported_name.clone()
              } else {
                let exported_name_str =
                  export_name(exported_name).expect("should have export_name");
                format!("{imported_name} as {exported_name_str}")
              }
            })
          })
          .collect::<Vec<_>>()
          .join(", "),
        match re_export_from {
          crate::chunk_link::ReExportFrom::Chunk(chunk_ukey) => {
            let chunk = compilation
              .build_chunk_graph_artifact
              .chunk_by_ukey
              .expect_get(chunk_ukey);
            Cow::Owned(format!("__RSPACK_ESM_CHUNK_{}", chunk.expect_id().as_str()))
          }
          crate::chunk_link::ReExportFrom::Request(request) => {
            Cow::Borrowed(request)
          }
        }
      )));
    }

    if let Some(default_export) = export_default {
      final_source.add(RawStringSource::from(format!(
        "export default {default_export};\n",
      )));
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
        replace_source.replace(start, end, &relative, None);
      }

      // concate module does this by render_module()
      // however esm module does not have concate module,
      // some replacement needs to be done here
      replace_source.boxed()
    } else {
      Arc::new(final_source)
    };

    let final_source = if replace_static_url {
      let content = final_source.source().into_string_lossy();
      let mut replace_source = ReplaceSource::new(final_source.clone());
      let replacement = URL_STATIC_PLACEHOLDER_RE
        .find_iter(&content)
        .map(|cap| (cap.start(), cap.end()));

      for (start, end) in replacement {
        let dep_id = &content[start + URL_STATIC_PLACEHOLDER.len()..end];
        let dep_id: DependencyId = dep_id
          .parse::<u32>()
          .unwrap_or_else(|_| panic!("should be valid dependency id \"{dep_id}\""))
          .into();
        let Some(module) = module_graph.module_identifier_by_dependency_id(&dep_id) else {
          continue;
        };
        let codegen_result = compilation.code_generation_results.get(module, None);
        let Some(filename) = codegen_result.data.get::<CodeGenerationDataFilename>() else {
          unreachable!()
        };

        replace_source.replace(start as u32, end as u32, filename.filename(), None);
      }

      // concate module does this by render_module()
      // however esm module does not have concate module,
      // some replacement needs to be done here
      replace_source.boxed()
    } else {
      final_source
    };

    Ok(Some(RenderSource {
      source: final_source,
    }))
  }

  pub async fn render_runtime(
    chunk_ukey: &ChunkUkey,
    compilation: &Compilation,
    runtime_requirements: RuntimeGlobals,
    runtime_template: &RuntimeCodeTemplate<'_>,
  ) -> Result<ConcatSource> {
    let module_factories: bool = runtime_requirements.contains(RuntimeGlobals::MODULE_FACTORIES);
    let require_function = runtime_requirements.contains(RuntimeGlobals::REQUIRE);
    let module_cache = runtime_requirements.contains(RuntimeGlobals::MODULE_CACHE);
    let intercept_module_execution =
      runtime_requirements.contains(RuntimeGlobals::INTERCEPT_MODULE_EXECUTION);
    let module_used = runtime_requirements.contains(RuntimeGlobals::MODULE);
    let require_scope_used = runtime_requirements.contains(RuntimeGlobals::REQUIRE_SCOPE);
    let use_require = require_function || intercept_module_execution || module_used;
    let mut source = ConcatSource::default();

    if use_require || module_cache {
      source.add(RawStringSource::from(format!(
        r#"// The module cache
var {} = {{}};
"#,
        runtime_template.render_runtime_variable(&RuntimeVariable::ModuleCache)
      )));
    }

    if use_require {
      source.add(RawStringSource::from(format!(
        r#"// The require function
function {}(moduleId) {{
"#,
        runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
      )));
      source.add(RawStringSource::from(
        JsPlugin::render_require(chunk_ukey, compilation, runtime_template).join("\n"),
      ));
      source.add(RawStringSource::from_static(
        r#"
}
"#,
      ));
    } else if require_scope_used {
      source.add(RawStringSource::from(format!(
        r#"// The require scope
var {} = {{}};
"#,
        runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
      )));
    }

    if module_factories {
      source.add(RawStringSource::from(format!(
        r#"// expose the modules object ({modules})
{module_factories} = {modules};
"#,
        module_factories =
          runtime_template.render_runtime_globals(&RuntimeGlobals::MODULE_FACTORIES),
        modules = runtime_template.render_runtime_variable(&RuntimeVariable::Modules),
      )));
    }

    if runtime_requirements.contains(RuntimeGlobals::MODULE_CACHE) {
      source.add(RawStringSource::from(format!(
        r#"// expose the module cache
{} = {};
"#,
        runtime_template.render_runtime_globals(&RuntimeGlobals::MODULE_CACHE),
        runtime_template.render_runtime_variable(&RuntimeVariable::ModuleCache),
      )));
    }

    if intercept_module_execution {
      source.add(RawStringSource::from(format!(
        r#"// expose the module execution interceptor
{} = [];
"#,
        runtime_template.render_runtime_globals(&RuntimeGlobals::INTERCEPT_MODULE_EXECUTION)
      )));
    }

    Ok(source)
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
            &name,
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
        source.replace(
          ident.id.span.real_lo(),
          ident.id.span.real_hi(),
          &name,
          None,
        );
      }
    }

    Ok(source)
  }

  pub fn render_external_required(
    root: ModuleIdentifier,
    compilation: &Compilation,
    chunk_link: &ChunkLinkContext,
    already_required: &mut IdentifierIndexSet,
    runtime_template: &RuntimeCodeTemplate<'_>,
  ) -> ConcatSource {
    let mut source = ConcatSource::default();

    for (id, interop_info) in &chunk_link.required {
      if !interop_info.from_module.contains(&root) {
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
