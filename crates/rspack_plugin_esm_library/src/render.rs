use std::{borrow::Cow, sync::Arc};

use rspack_collections::{IdentifierMap, UkeyMap, UkeySet};
use rspack_core::{
  find_new_name, get_js_chunk_filename_template, property_access,
  rspack_sources::{ConcatSource, RawSource, RawStringSource},
  AssetInfo, BoxModule, Chunk, ChunkGraph, ChunkUkey, Compilation, ConcatenationScope, ModuleInfo,
  PathData, PathInfo, RuntimeGlobals, SourceType, SpanExt,
};
use rspack_error::{error, Result};
use rspack_plugin_javascript::{
  render_bootstrap,
  runtime::{render_module, render_runtime_modules},
  RenderSource,
};
use rspack_util::{
  atom::Atom,
  fx_hash::{FxHashMap, FxIndexSet},
};

#[inline]
fn get_chunk(compilation: &Compilation, chunk_ukey: ChunkUkey) -> &Chunk {
  compilation.chunk_by_ukey.expect_get(&chunk_ukey)
}

use crate::{dependency::dyn_import::NAMESPACE_SYMBOL, EsmLibraryPlugin};
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
    _asset_info: &mut AssetInfo,
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
      if !concatenated_modules.contains(id) {
        decl_modules.push(*m);
      }
    }

    // sort by module identifier to get better gzip size, as similar identifiers
    // means more probably they are in the same directory, and the code is more
    // likely to be similar.
    decl_modules.sort_by_key(|m| m.identifier());

    // find import
    let mut decl_source = ConcatSource::default();

    let chunk = get_chunk(compilation, *chunk_ukey);

    let filename_template = get_js_chunk_filename_template(
      chunk,
      &compilation.options.output,
      &compilation.chunk_group_by_ukey,
    );
    let mut asset_info = AssetInfo::default();
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
        &mut asset_info,
      )
      .await?;

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
    let mut imported_symbols = IdentifierMap::<FxHashMap<Atom, Atom>>::default();
    let mut imported_chunks = UkeyMap::<ChunkUkey, FxIndexSet<(Atom, Atom)>>::default();

    // we use webpack_require to load modules that are not scope hoisted
    // and we should also deconflict them
    // const symbol = __webpack_require__('./main.js')
    let mut required_symbols = IdentifierMap::<Atom>::default();

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

      if chunk_runtime_requirements.contains(RuntimeGlobals::REQUIRE) {
        // should expose the __webpack_require__
        export_specifiers.push(RuntimeGlobals::REQUIRE.name());
      }
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

    let mut used_names = chunk_link.used_names.clone();
    for m in &chunk_link.hoisted_modules {
      let info = concatenated_modules_map
        .get(m)
        .expect("should have info")
        .as_concatenated();

      let mut source = info.source.clone().expect("should have source");

      for ((atom, ctxt), refs) in &info.binding_to_ref {
        if ctxt == &info.global_ctxt
          && let Some(match_module_ref) = ConcatenationScope::match_module_reference(atom)
        {
          let final_name = chunk_link
            .ref_to_final_name
            .get(atom.as_str())
            .expect("should already set");

          let final_name = match final_name {
            rspack_core::ModuleReference::Binding(binding) => {
              let (reference, is_property_access) = match binding {
                rspack_core::Binding::Raw(raw_binding) => {
                  let imported_id = raw_binding.info_id;
                  let is_property_access = !raw_binding.ids.is_empty();
                  let symbol = &raw_binding.raw_name;

                  let needs_require_module = matches!(
                    concatenated_modules_map
                      .get(&imported_id)
                      .expect("should have info"),
                    ModuleInfo::External(_)
                  );

                  // add required symbols
                  let local = if let Some(local) = required_symbols.get(&imported_id) {
                    local.clone()
                  } else if used_names.contains(symbol) {
                    let local = find_new_name(symbol, &chunk_link.used_names, None, "");
                    used_names.insert(local.clone());
                    local
                  } else {
                    used_names.insert(symbol.clone());
                    symbol.clone()
                  };

                  if needs_require_module {
                    required_symbols.insert(imported_id, local.clone());
                  }

                  let reference = format!(
                    "{}{}{}",
                    &local,
                    raw_binding.comment.clone().unwrap_or_default(),
                    property_access(&raw_binding.ids, 0)
                  );

                  let runtime_chunk = Self::get_runtime_chunk(*chunk_ukey, compilation);
                  let require_symbol: Atom = RuntimeGlobals::REQUIRE.name().into();

                  imported_chunks.insert(
                    runtime_chunk,
                    std::iter::once((require_symbol.clone(), require_symbol)).collect(),
                  );
                  (reference, is_property_access)
                }

                rspack_core::Binding::Symbol(symbol_binding) => {
                  let imported = imported_symbols.entry(symbol_binding.info_id).or_default();
                  let info = concatenated_modules_map
                    .get(&symbol_binding.info_id)
                    .expect("info should be set in finish_modules")
                    .as_concatenated();
                  let symbol = info
                    .internal_names
                    .get(&symbol_binding.name)
                    .expect("should have set top level symbol");

                  let local = if let Some(local) = imported.get(symbol) {
                    local.clone()
                  } else if used_names.contains(symbol) {
                    let local = find_new_name(symbol, &chunk_link.used_names, None, "");
                    used_names.insert(local.clone());
                    imported.insert(symbol.clone(), local.clone());
                    local
                  } else {
                    used_names.insert(symbol.clone());
                    imported.insert(symbol.clone(), symbol.clone());
                    symbol.clone()
                  };

                  let is_property_access = symbol_binding.ids.len() > 1;

                  let reference = format!(
                    "{}{}{}",
                    &local,
                    symbol_binding.comment.clone().unwrap_or_default(),
                    property_access(&symbol_binding.ids, 0)
                  );
                  (reference, is_property_access)
                }
              };

              let final_name =
                if is_property_access && match_module_ref.call && !match_module_ref.direct_import {
                  if match_module_ref.asi_safe.unwrap_or_default() {
                    format!("(0,{reference})")
                  } else if let Some(_asi_safe) = match_module_ref.asi_safe {
                    format!(";(0,{reference})")
                  } else {
                    format!("/*#__PURE__*/Object({reference})")
                  }
                } else {
                  reference
                };

              Cow::Owned(final_name)
            }
            rspack_core::ModuleReference::Str(final_name) => Cow::Borrowed(final_name),
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
          for ref_atom in refs {
            let content = if already_in_chunk {
              Cow::Borrowed(internal_symbol.as_str())
            } else {
              Cow::Owned(format!("{NAMESPACE_SYMBOL}.{internal_symbol}"))
            };
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
        let require_symbol: Atom = RuntimeGlobals::REQUIRE.name().into();
        imported_chunks.insert(
          runtime_chunk,
          std::iter::once((require_symbol.clone(), require_symbol)).collect(),
        );
      }
    }

    for (module_id, imported) in imported_symbols.iter() {
      let ref_chunk = Self::get_module_chunk(*module_id, compilation);

      if &ref_chunk == chunk_ukey {
        continue;
      }

      let imported_atoms = imported_chunks.entry(ref_chunk).or_default();
      imported_atoms.extend(imported.clone().into_iter());
    }

    if !imported_chunks.is_empty() {
      for (chunk, imported) in &imported_chunks {
        final_source.add(RawStringSource::from(format!(
          "import {{ {} }} from \"__RSPACK_ESM_CHUNK_{}\";\n",
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
            .join(", "),
          chunk.as_u32()
        )));
      }

      final_source.add(RawSource::from_static("\n"));
    }

    final_source.add(decl_source);

    // render __webpack_require__ call after decl modules
    if !required_symbols.is_empty() {
      let mut already_imported_chunks: UkeySet<ChunkUkey> = imported_symbols
        .keys()
        .map(|module_id| Self::get_module_chunk(*module_id, compilation))
        .collect();
      let mut extra_imports = FxIndexSet::default();

      let mut required_symbols = required_symbols.iter().collect::<Vec<_>>();

      required_symbols.sort_by(|(a, _), (b, _)| {
        module_graph
          .get_post_order_index(a)
          .cmp(&module_graph.get_post_order_index(b))
      });

      let required_str = RawStringSource::from(
        required_symbols
          .into_iter()
          .map(|(id, atom)| {
            let ref_chunk = Self::get_module_chunk(*id, compilation);
            if &ref_chunk != chunk_ukey && !already_imported_chunks.contains(&ref_chunk) {
              already_imported_chunks.insert(ref_chunk);
              extra_imports.insert(format!(
                "import \"__RSPACK_ESM_CHUNK_{}\";\n",
                ref_chunk.as_u32()
              ));
            }

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

      final_source.add(RawStringSource::from(
        extra_imports.into_iter().collect::<Vec<_>>().join("\n"),
      ));
      final_source.add(RawSource::from_static("\n"));
      final_source.add(required_str);
    }

    final_source.add(render_source);

    for (id, exports) in &chunk_link.exports {
      let info = concatenated_modules_map
        .get(id)
        .expect("should have info")
        .as_concatenated();
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

    if !export_specifiers.is_empty() {
      final_source.add(RawStringSource::from(format!(
        "export {{ {} }};",
        export_specifiers.join(", ")
      )));
    }

    Ok(Some(RenderSource {
      source: Arc::new(final_source),
    }))
  }
}
