use std::{borrow::Cow, sync::Arc};

use rspack_collections::{IdentifierMap, UkeyIndexMap, UkeySet};
use rspack_core::{
  AssetInfo, BoxModule, Chunk, ChunkGraph, ChunkLinkContext, ChunkRenderContext, ChunkUkey,
  Compilation, ConcatenatedModuleInfo, ConcatenationScope, DEFAULT_EXPORT, InitFragment,
  ModuleIdentifier, ModuleInfo, PathData, PathInfo, Ref, RuntimeGlobals, SourceType, SpanExt,
  get_js_chunk_filename_template, render_init_fragments,
  rspack_sources::{BoxSource, ConcatSource, RawSource, RawStringSource, ReplaceSource, SourceExt},
};
use rspack_error::{Result, error};
use rspack_plugin_javascript::{
  JsPlugin, RenderSource,
  runtime::{render_module, render_runtime_modules},
};
use rspack_util::{
  atom::Atom,
  fx_hash::{FxHashMap, FxIndexSet},
};

#[inline]
fn get_chunk(compilation: &Compilation, chunk_ukey: ChunkUkey) -> &Chunk {
  compilation.chunk_by_ukey.expect_get(&chunk_ukey)
}

use crate::{EsmLibraryPlugin, dependency::dyn_import::NAMESPACE_SYMBOL};

impl EsmLibraryPlugin {
  pub(crate) fn get_runtime_chunk(chunk_ukey: ChunkUkey, compilation: &Compilation) -> ChunkUkey {
    let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
    let group = chunk
      .groups()
      .iter()
      .next()
      .expect("should have at least one group");
    let group = compilation.chunk_group_by_ukey.expect_get(group);
    let mut stack = vec![group];
    let mut visited = UkeySet::default();

    while let Some(group) = stack.pop() {
      if !visited.insert(group.ukey) {
        continue;
      }

      if group.kind.is_entrypoint() {
        return group.get_runtime_chunk(&compilation.chunk_group_by_ukey);
      }

      stack.extend(
        group
          .parents_iterable()
          .map(|group| compilation.chunk_group_by_ukey.expect_get(group)),
      );
    }

    unreachable!("chunk should have at least one ancestor that is entrypoint")
  }

  pub(crate) async fn render_chunk(
    &self,
    compilation: &Compilation,
    chunk_ukey: &ChunkUkey,
    asset_info: &mut AssetInfo,
  ) -> Result<Option<RenderSource>> {
    let module_graph = compilation.get_module_graph();
    let chunk_link = compilation
      .chunk_graph
      .link
      .as_ref()
      .expect("should have set link info")
      .get(chunk_ukey)
      .expect("should have chunk");

    // modules that can be concatenated
    let concatenated_modules = &chunk_link.hoisted_modules;

    let mut decl_modules: Vec<&BoxModule> = Default::default();
    let mut chunk_init_fragments: Vec<Box<dyn InitFragment<ChunkRenderContext> + 'static>> = vec![];

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
    let chunk = get_chunk(compilation, *chunk_ukey);
    let filename_template = get_js_chunk_filename_template(
      chunk,
      &compilation.options.output,
      &compilation.chunk_group_by_ukey,
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
        asset_info,
      )
      .await?;

    for (id, m) in &chunk_modules {
      if !concatenated_modules.contains(id) {
        decl_modules.push(*m);
      }

      let info = concatenated_modules_map.get(id).expect("should have info");
      if let Some(info) = info.try_as_concatenated() {
        chunk_init_fragments.extend(info.chunk_init_fragments.clone());
      }
    }

    // sort by module identifier to get better gzip size, as similar identifiers
    // means more probably they are in the same directory, and the code is more
    // likely to be similar.
    decl_modules.sort_by_key(|m| m.identifier());

    // modules that are not scope hoisted, store in runtime
    let mut decl_source = ConcatSource::default();

    if !decl_modules.is_empty() {
      // use Object.assign to register module to __webpack_modules__ object
      // so that other module can use __webpack_require__ to load it
      // Object.assign(__webpack_require__.m, { "./src/main.js"(require, exports) { ... } })
      decl_source.add(RawSource::from(format!(
        "Object.assign({}, {{\n",
        RuntimeGlobals::MODULE_FACTORIES
      )));

      for m in &decl_modules {
        let Some((module_source, init_frags, _)) =
          render_module(compilation, chunk_ukey, m, true, true, &output_path).await?
        else {
          continue;
        };
        chunk_init_fragments.extend(init_frags);
        decl_source.add(module_source.clone());
      }

      decl_source.add(RawSource::from_static("});\n"));
    }

    // present as
    // a.js -> (imported symbol, local symbol)
    // we use webpack_require to load modules that are not scope hoisted
    // and we should also deconflict them
    // const symbol = __webpack_require__('./main.js')

    // render cross module links
    let mut runtime_source = ConcatSource::default();
    let mut render_source = ConcatSource::default();
    let mut export_specifiers: FxIndexSet<Cow<str>> = chunk_link
      .static_exports
      .iter()
      .map(|s| Cow::Borrowed(s.as_str()))
      .collect();

    // render webpack runtime
    if chunk.has_runtime(&compilation.chunk_group_by_ukey) {
      let chunk_runtime_requirements =
        ChunkGraph::get_chunk_runtime_requirements(compilation, chunk_ukey);
      if chunk_runtime_requirements.contains(RuntimeGlobals::MODULE_FACTORIES) {
        runtime_source.add(RawStringSource::from_static(
          "\nvar __webpack_modules__ = {};\n",
        ));
      }
      let runtimes = Self::render_runtime(chunk_ukey, compilation).await?;

      runtime_source.add(runtimes);
      runtime_source.add(RawSource::from("\n"));
      runtime_source.add(render_runtime_modules(compilation, chunk_ukey).await?);
      runtime_source.add(RawSource::from("\n"));
    }

    // render namespace object before render module contents
    for namespace in chunk_link.namespace_object_sources.values() {
      render_source.add(RawStringSource::from(format!("{namespace}\n")));
    }

    for m in &chunk_link.hoisted_modules {
      let info = concatenated_modules_map.get(m).expect("should have info");
      let info = info.as_concatenated();

      let source = Self::render_module(info, chunk_link)?;

      if matches!(compilation.options.output.pathinfo, PathInfo::Bool(false)) {
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

      render_source.add(Self::render_external_required(*m, compilation, chunk_link));
      render_source.add(source);
      render_source.add(RawSource::from_static("\n"));
    }

    // render imports and exports to other chunks
    let mut final_source = ConcatSource::default();

    let runtime_requirements = ChunkGraph::get_chunk_runtime_requirements(compilation, chunk_ukey);
    if !runtime_requirements.is_empty() {
      let runtime_chunk = Self::get_runtime_chunk(*chunk_ukey, compilation);
      if &runtime_chunk != chunk_ukey {
        final_source.add(RawStringSource::from(format!(
          "import {{ {} }} from \"__RSPACK_ESM_CHUNK_{}\";\n",
          RuntimeGlobals::REQUIRE,
          runtime_chunk.as_u32()
        )));
      }
    }

    if !chunk_link.imports.is_empty() {
      let mut imported_chunks = UkeyIndexMap::<ChunkUkey, FxHashMap<Atom, Atom>>::default();
      for (id, imports) in &chunk_link.imports {
        let chunk = Self::get_module_chunk(*id, compilation);
        if imports.is_empty() {
          continue;
        }

        let info = concatenated_modules_map
          .get(id)
          .expect("should have info")
          .as_concatenated();
        let imported_symbols = imported_chunks.entry(chunk).or_default();
        for (imported, local) in imports {
          let internal_symbol = info
            .internal_names
            .get(imported)
            .unwrap_or_else(|| panic!("module {id} should have internal name for {imported}"));
          imported_symbols.insert(internal_symbol.clone(), local.clone());
        }
      }

      for (chunk, imported) in &imported_chunks {
        final_source.add(RawStringSource::from(format!(
          "import {}\"__RSPACK_ESM_CHUNK_{}\";\n",
          if imported.is_empty() {
            String::new()
          } else {
            format!(
              "{{ {} }} from ",
              imported
                .iter()
                .map(|(imported, local)| {
                  if imported == local {
                    Cow::Borrowed(imported.as_str())
                  } else {
                    Cow::Owned(format!("{imported} as {local}"))
                  }
                })
                .collect::<Vec<_>>()
                .join(", ")
            )
          },
          chunk.as_u32()
        )));
      }

      final_source.add(RawSource::from_static("\n"));
    }

    final_source.add(runtime_source);
    final_source.add(decl_source);
    final_source.add(render_source);

    let mut already_export_default = false;
    for (id, exports) in &chunk_link.exports {
      let info = concatenated_modules_map.get(id).expect("should have info");

      match info {
        ModuleInfo::Concatenated(info) => {
          for raw_symbol in exports {
            let is_default = raw_symbol.as_str() == "default";
            let symbol = if is_default {
              DEFAULT_EXPORT.into()
            } else {
              raw_symbol.clone()
            };
            let Some(local_symbol) = info.get_internal_name(&symbol) else {
              return Err(error!(
                "module {id} should already set internal names for export: {symbol}, internal_names: {:?}",
                &info.internal_names
              ));
            };

            if is_default && !already_export_default {
              export_specifiers.insert(Cow::Owned(format!("{local_symbol} as default")));
              already_export_default = true;
            } else {
              export_specifiers.insert(Cow::Borrowed(local_symbol.as_str()));
            }
          }
        }
        ModuleInfo::External(_info) => {
          // export from external module
          // const ns = __webpack_require__('module')
          // export { ns.foo as foo }
          // for export in exports {}
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

    // if the entry is wrapped, we should start from entry
    for entry_module in compilation.chunk_graph.get_chunk_entry_modules(chunk_ukey) {
      if matches!(
        concatenated_modules_map
          .get(&entry_module)
          .expect("should have module info"),
        ModuleInfo::External(_)
      ) {
        final_source.add(RawStringSource::from(format!(
          "{}({});\n",
          RuntimeGlobals::REQUIRE,
          serde_json::to_string(
            ChunkGraph::get_module_id(&compilation.module_ids_artifact, entry_module)
              .expect("should set module id")
              .as_str()
          )
          .expect("should serialize module id to string")
        )));
      }
    }

    // render init fragments
    let final_source = render_init_fragments(
      final_source.boxed(),
      chunk_init_fragments,
      &mut ChunkRenderContext {},
    )?;

    Ok(Some(RenderSource {
      source: Arc::new(final_source),
    }))
  }
  pub async fn render_runtime<'me>(
    chunk_ukey: &ChunkUkey,
    compilation: &'me Compilation,
  ) -> Result<ConcatSource> {
    let runtime_requirements = ChunkGraph::get_chunk_runtime_requirements(compilation, chunk_ukey);
    let module_factories = runtime_requirements.contains(RuntimeGlobals::MODULE_FACTORIES);
    let require_function = runtime_requirements.contains(RuntimeGlobals::REQUIRE);
    let module_cache = runtime_requirements.contains(RuntimeGlobals::MODULE_CACHE);
    let intercept_module_execution =
      runtime_requirements.contains(RuntimeGlobals::INTERCEPT_MODULE_EXECUTION);
    let module_used = runtime_requirements.contains(RuntimeGlobals::MODULE);
    let require_scope_used = runtime_requirements.contains(RuntimeGlobals::REQUIRE_SCOPE);
    let use_require = require_function || intercept_module_execution || module_used;
    let mut source = ConcatSource::default();

    if use_require || module_cache {
      source.add(RawSource::from_static(
        "// The module cache\nvar __webpack_module_cache__ = {};\n",
      ));
    }

    if use_require {
      source.add(RawStringSource::from(format!(
        "// The require function\nfunction {}(moduleId) {{\n",
        RuntimeGlobals::REQUIRE
      )));
      source.add(RawStringSource::from(
        JsPlugin::render_require(chunk_ukey, compilation).join("\n"),
      ));
      source.add(RawSource::from_static("\n}\n"));
    } else if require_scope_used {
      source.add(RawStringSource::from(format!(
        "// The require scope\nvar {} = {{}};\n",
        RuntimeGlobals::REQUIRE
      )));
    }

    if module_factories || runtime_requirements.contains(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY)
    {
      source.add(RawStringSource::from(format!(
        "// expose the modules object (__webpack_modules__)\n{} = __webpack_modules__;\n",
        RuntimeGlobals::MODULE_FACTORIES
      )));
    }

    if runtime_requirements.contains(RuntimeGlobals::MODULE_CACHE) {
      source.add(RawStringSource::from(format!(
        "// expose the module cache\n{} = __webpack_module_cache__;\n",
        RuntimeGlobals::MODULE_CACHE
      )));
    }

    if intercept_module_execution {
      source.add(RawStringSource::from(format!(
        "// expose the module execution interceptor\n{} = [];\n",
        RuntimeGlobals::INTERCEPT_MODULE_EXECUTION
      )));
    }

    Ok(source)
  }

  pub fn render_module(
    info: &ConcatenatedModuleInfo,
    chunk_link: &ChunkLinkContext,
  ) -> Result<ReplaceSource<BoxSource>> {
    let mut source = info.source.clone().expect("should have source");

    for ((atom, ctxt), refs) in &info.binding_to_ref {
      if ctxt == &info.global_ctxt && ConcatenationScope::is_module_reference(atom) {
        let binding_ref = chunk_link
          .refs
          .get(atom.as_str())
          .unwrap_or_else(|| panic!("should already set ref for atom {atom}"));

        let final_name = match binding_ref {
          Ref::Symbol(symbol_ref) => Cow::Owned(symbol_ref.render()),
          Ref::Inline(inline) => Cow::Borrowed(inline),
        };

        for ident in refs {
          source.replace(
            ident.id.span.real_lo(),
            ident.id.span.real_hi() + 2,
            &final_name,
            None,
          );
        }
      } else if ctxt == &info.global_ctxt && ConcatenationScope::is_dynamic_module_reference(atom) {
        let (in_same_chunk, binding_ref) = chunk_link
          .dyn_refs
          .get(atom.as_str())
          .unwrap_or_else(|| panic!("should already set dynamic ref for atom {atom}"));

        let final_name = match binding_ref {
          Ref::Symbol(symbol_ref) => Cow::Owned(symbol_ref.render()),
          Ref::Inline(inline) => Cow::Borrowed(inline),
        };

        let final_name = if *in_same_chunk {
          final_name.into_owned()
        } else {
          format!("{NAMESPACE_SYMBOL}.{final_name}")
        };

        for ref_atom in refs {
          source.replace(
            ref_atom.id.span.real_lo(),
            ref_atom.id.span.real_hi(),
            final_name.as_str(),
            None,
          );
        }
      }
    }

    for ident in &info.idents {
      if ident.id.ctxt != info.module_ctxt {
        continue;
      }

      if let Some(internal_name) = info.internal_names.get(&ident.id.sym) {
        source.replace(
          ident.id.span.real_lo(),
          ident.id.span.real_hi(),
          internal_name,
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
  ) -> ConcatSource {
    let mut source = ConcatSource::default();

    for (id, interop_info) in &chunk_link.required {
      if interop_info.from_module != root {
        continue;
      }

      let name = interop_info.required_symbol.as_ref();

      if let Some(name) = name {
        source.add(RawStringSource::from(format!(
          "const {name} = __webpack_require__({});\n",
          serde_json::to_string(
            ChunkGraph::get_module_id(&compilation.module_ids_artifact, *id)
              .expect("should set module id")
              .as_str()
          )
          .expect("module id to string should success")
        )));

        if let Some(namespace_object) = &interop_info.namespace_object {
          source.add(RawStringSource::from(format!(
            "var {} = /*#__PURE__*/{}({}, 2);\n",
            namespace_object,
            RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT,
            name
          )));
        }

        if let Some(namespace_object) = &interop_info.namespace_object2 {
          source.add(RawStringSource::from(format!(
            "var {} = /*#__PURE__*/{}({});\n",
            namespace_object,
            RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT,
            name
          )));
        }

        if let Some(default_access) = &interop_info.default_access {
          source.add(RawStringSource::from(format!(
            "var {} = /*#__PURE__*/{}({});\n",
            default_access,
            RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT,
            name
          )));
        }
      } else {
        source.add(RawStringSource::from(format!(
          "__webpack_require__({});\n",
          serde_json::to_string(
            ChunkGraph::get_module_id(&compilation.module_ids_artifact, *id)
              .expect("should set module id")
              .as_str()
          )
          .expect("module id to string should success")
        )));
      }
    }

    source
  }
}
