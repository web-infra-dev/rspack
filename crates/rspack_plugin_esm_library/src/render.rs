use std::{borrow::Cow, sync::Arc};

use rspack_collections::{IdentifierMap, UkeyIndexMap, UkeySet};
use rspack_core::{
  AssetInfo, BoxModule, Chunk, ChunkGraph, ChunkRenderContext, ChunkUkey, Compilation,
  ConcatenationScope, InitFragment, ModuleInfo, PathData, PathInfo, Ref, RuntimeGlobals,
  SourceType, SpanExt, get_js_chunk_filename_template, render_init_fragments,
  rspack_sources::{ConcatSource, RawSource, RawStringSource, SourceExt},
};
use rspack_error::{Result, error};
use rspack_plugin_javascript::{
  RenderSource, render_bootstrap,
  runtime::{render_module, render_runtime_modules},
};
use rspack_util::{atom::Atom, fx_hash::FxHashMap};

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
        let Some((module_source, _, _)) =
          render_module(compilation, chunk_ukey, m, true, true, &output_path).await?
        else {
          continue;
        };
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
    let mut render_source = ConcatSource::default();
    let mut export_specifiers = vec![];

    // render webpack runtime
    if chunk.has_runtime(&compilation.chunk_group_by_ukey) {
      let chunk_runtime_requirements =
        ChunkGraph::get_chunk_runtime_requirements(compilation, chunk_ukey);
      if chunk_runtime_requirements.contains(RuntimeGlobals::MODULE_FACTORIES) {
        render_source.add(RawStringSource::from_static(
          "\nvar __webpack_modules__ = {};\n",
        ));
      }
      let bootstrap = render_bootstrap(chunk_ukey, compilation).await?;
      render_source.add(RawStringSource::from(bootstrap.header.join("\n")));
      render_source.add(RawSource::from("\n"));
      render_source.add(RawStringSource::from(bootstrap.startup.join("\n")));
      render_source.add(render_runtime_modules(compilation, chunk_ukey).await?);
      render_source.add(RawSource::from("\n"));

      if chunk_runtime_requirements.contains(RuntimeGlobals::EXPORTS) {
        render_source.add(RawStringSource::from_static(
          "var __webpack_exports__ = {};\n",
        ));
      }

      // should expose the __webpack_require__
      export_specifiers.push(RuntimeGlobals::REQUIRE.name());
    }

    // render namespace object before render module contents
    for m in &chunk_link.hoisted_modules {
      let info = concatenated_modules_map
        .get(m)
        .expect("should have info")
        .as_concatenated();
      if let Some(namespace) = &info.namespace_object_source {
        render_source.add(RawStringSource::from(format!("{namespace}\n")));
      }
    }

    for m in &chunk_link.hoisted_modules {
      let info = concatenated_modules_map
        .get(m)
        .expect("should have info")
        .as_concatenated();

      let mut source = info.source.clone().expect("should have source");

      for ((atom, ctxt), refs) in &info.binding_to_ref {
        if ctxt == &info.global_ctxt && ConcatenationScope::is_module_reference(atom) {
          let binding_ref = chunk_link
            .refs
            .get(atom.as_str())
            .expect("should already set");

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
        } else if ctxt == &info.global_ctxt
          && let Some((index, already_in_chunk, atom)) =
            ConcatenationScope::match_dynamic_module_reference(atom)
        {
          let (ref_module, ref_info) = concatenated_modules_map
            .get_index(index)
            .expect("should have module");
          let internal_names = &ref_info.as_concatenated().internal_names;
          let internal_symbol = internal_names.get(&atom).unwrap_or_else(|| {
            panic!(
              "module {} should have set internal name for: {}, internal_names: {:?}",
              ref_module, atom, &internal_names
            )
          });
          let content = if already_in_chunk {
            Cow::Borrowed(internal_symbol.as_str())
          } else {
            Cow::Owned(format!("{NAMESPACE_SYMBOL}.{internal_symbol}"))
          };
          for ref_atom in refs {
            source.replace(
              ref_atom.id.span.real_lo(),
              ref_atom.id.span.real_hi(),
              &content,
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
            .unwrap_or_else(|| panic!("module {} should have internal name for {}", id, imported));
          imported_symbols.insert(internal_symbol.clone(), local.clone());
        }
      }

      for (chunk, imported) in &imported_chunks {
        final_source.add(RawStringSource::from(format!(
          "import {}\"__RSPACK_ESM_CHUNK_{}\";\n",
          if imported.is_empty() {
            "".into()
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

    final_source.add(decl_source);

    // render __webpack_require__ call after decl modules
    if !chunk_link.required.is_empty() {
      let required_str = RawStringSource::from(
        chunk_link
          .required
          .iter()
          .map(|(id, atom)| {
            format!(
              "const {} = __webpack_require__({});\n",
              atom,
              serde_json::to_string(
                ChunkGraph::get_module_id(&compilation.module_ids_artifact, *id)
                  .expect("should set module id")
                  .as_str()
              )
              .expect("module id to string should success")
            )
          })
          .collect::<Vec<_>>()
          .join("\n"),
      );
      final_source.add(RawSource::from_static("\n"));
      final_source.add(required_str);
    }

    final_source.add(render_source);

    for (id, exports) in &chunk_link.exports {
      let info = concatenated_modules_map.get(id).expect("should have info");

      match info {
        ModuleInfo::Concatenated(info) => {
          for symbol in exports {
            let Some(local_symbol) = info.internal_names.get(symbol) else {
              return Err(error!(
                "module {id} should already set internal names for export: {symbol}, internal_names: {:?}",
                &info.internal_names
              ));
            };

            export_specifiers.push(local_symbol.as_str());
          }
        }
        ModuleInfo::External(info) => {
          // export from external module
          // const ns = __webpack_require__('module')
          // export { ns.foo as foo }
          for export in exports {}
        }
      }
    }

    if !export_specifiers.is_empty() {
      final_source.add(RawStringSource::from(format!(
        "export {{ {} }};",
        export_specifiers.join(", ")
      )));
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
}
