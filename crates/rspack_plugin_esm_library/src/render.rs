use std::{borrow::Cow, collections::hash_map::Entry, sync::Arc};

use rspack_collections::{IdentifierIndexSet, IdentifierMap, UkeyIndexMap, UkeyMap, UkeySet};
use rspack_core::{
  AssetInfo, BoxModule, Chunk, ChunkGraph, ChunkUkey, Compilation, ConcatenatedModule,
  ConcatenatedModuleIdent, ConcatenatedModuleInfo, ConcatenationScope, IdentCollector, ModuleInfo,
  NAMESPACE_OBJECT_EXPORT, PathData, PathInfo, RuntimeGlobals, SourceType, SpanExt, find_new_name,
  get_js_chunk_filename_template, property_access,
  reserved_names::RESERVED_NAMES,
  rspack_sources::{ConcatSource, RawSource, RawStringSource, ReplaceSource},
};
use rspack_error::Result;
use rspack_javascript_compiler::ast::Ast;
use rspack_plugin_javascript::{
  RenderSource, render_bootstrap, render_require,
  runtime::{render_chunk_runtime_modules, render_module, render_runtime_modules},
  visitors::swc_visitor::resolver,
};
use rspack_util::{
  atom::Atom,
  fx_hash::{FxHashMap, FxHashSet, FxIndexSet},
};

use crate::EsmLibraryPlugin;
impl EsmLibraryPlugin {
  pub(crate) fn get_runtime_chunk(chunk_ukey: ChunkUkey, compilation: &Compilation) -> ChunkUkey {
    let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
    let group = chunk.groups().into_iter().next().unwrap();
    let group = compilation.chunk_group_by_ukey.expect_get(group);
    group.get_runtime_chunk(&compilation.chunk_group_by_ukey)
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
      .unwrap()
      .get(chunk_ukey)
      .unwrap();

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

    decl_modules.sort_by_key(|m| m.identifier());
    // find import
    let mut render_source = ConcatSource::default();

    let chunk = compilation.chunk_by_ukey.get(chunk_ukey).unwrap();
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
      render_source.add(RawSource::from(format!(
        "Object.assign({}, {{\n",
        RuntimeGlobals::MODULE_FACTORIES
      )));

      for m in &decl_modules {
        let codegen_res = compilation
          .code_generation_results
          .get(&m.identifier(), None);

        let Some((module_source, _, _)) =
          render_module(compilation, chunk_ukey, m, true, true, &output_path).await?
        else {
          continue;
        };
        render_source.add(module_source.clone());
      }

      render_source.add(RawSource::from_static("});\n"));
    }

    for m in decl_modules {
      let ModuleInfo::External(info) = concatenated_modules_map
        .get(&m.identifier())
        .expect("should have info")
      else {
        unreachable!("should be external module")
      };

      // render_source.add(RawStringSource::from(format!(
      //   "var {} = {}({});\n",
      //   info.name.as_ref().expect("should have name"),
      //   RuntimeGlobals::REQUIRE,
      //   serde_json::to_string(
      //     ChunkGraph::get_module_id(&compilation.module_ids_artifact, info.module)
      //       .expect("should have module id")
      //   )
      //   .expect("should json stringify module id")
      // )));
    }

    // present as
    // a.js -> (imported symbol, local symbol)
    let mut imported_symbols = IdentifierMap::<FxHashMap<Atom, Atom>>::default();
    let mut imported_chunks = UkeyMap::<ChunkUkey, FxIndexSet<(Atom, Atom)>>::default();

    // const symbol = __webpack_require__('./main.js')
    let mut required_symbols = IdentifierMap::<Atom>::default();

    // render cross module links
    for m in &chunk_link.hoisted_modules {
      let info = concatenated_modules_map
        .get(m)
        .expect("should have info")
        .as_concatenated();

      let mut source = info.source.clone().unwrap();
      let codegen_res = compilation.code_generation_results.get(m, None);
      let mut used_names = info.all_used_names.clone();

      for ((atom, ctxt), refs) in &info.binding_to_ref {
        if let Some(match_module_ref) = ConcatenationScope::match_module_reference(&atom) {
          let final_name = chunk_link
            .ref_to_final_name
            .get(atom.as_str())
            .expect("should already set");

          match final_name {
            rspack_core::ModuleReference::Binding(binding) => {
              let (reference, is_property_access) = match binding {
                rspack_core::Binding::Raw(raw_binding) => {
                  let imported_id = raw_binding.info_id;
                  let is_property_access = !raw_binding.ids.is_empty();
                  let symbol = &raw_binding.raw_name;
                  let local = if let Some(local) = required_symbols.get(&imported_id) {
                    local.clone()
                  } else if used_names.contains(symbol) {
                    let local = find_new_name(&symbol, &chunk_link.used_names, None, "");
                    used_names.insert(local.clone());
                    required_symbols.insert(imported_id, local.clone());
                    local
                  } else {
                    used_names.insert(symbol.clone());
                    required_symbols.insert(imported_id, symbol.clone());
                    symbol.clone()
                  };

                  let ref_chunk = Self::get_module_chunk(imported_id, compilation);
                  let reference = format!(
                    "{}{}{}",
                    &local,
                    raw_binding.comment.clone().unwrap_or_default(),
                    property_access(&raw_binding.ids, 0)
                  );

                  let runtime_chunk = Self::get_runtime_chunk(*chunk_ukey, compilation);
                  let require_symbol: swc_core::atoms::Atom = RuntimeGlobals::REQUIRE.name().into();

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
                    .unwrap()
                    .as_concatenated();
                  let symbol = info.internal_names.get(&symbol_binding.name).unwrap();

                  let local = if let Some(local) = imported.get(symbol) {
                    local.clone()
                  } else if used_names.contains(symbol) {
                    let local = find_new_name(&symbol, &chunk_link.used_names, None, "");
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

              for ident in refs {
                source.replace(
                  ident.id.span.real_lo(),
                  ident.id.span.real_hi() + 2,
                  &final_name,
                  None,
                );
              }
            }
            rspack_core::ModuleReference::Str(final_name) => {
              for ident in refs {
                source.replace(
                  ident.id.span.real_lo(),
                  ident.id.span.real_hi() + 2,
                  final_name,
                  None,
                );
              }
            }
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
          ChunkGraph::get_module_id(&compilation.module_ids_artifact, *m)
            .map(|id| { id.to_string() })
            .unwrap_or_else(|| {
              let module = module_graph.module_by_identifier(m).unwrap();
              module
                .readable_identifier(&compilation.options.context)
                .to_string()
            })
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
        let require_symbol: swc_core::atoms::Atom = RuntimeGlobals::REQUIRE.name().into();
        imported_chunks.insert(
          runtime_chunk,
          std::iter::once((require_symbol.clone(), require_symbol)).collect(),
        );
      }
    }

    imported_symbols.iter().for_each(|(module_id, imported)| {
      let ref_chunk = Self::get_module_chunk(*module_id, compilation);
      let imported_atoms = imported_chunks.entry(ref_chunk).or_default();
      imported_atoms.extend(imported.clone().into_iter());
    });

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
                Cow::Owned(format!("{} as {}", imported, local))
              }
            })
            .collect::<Vec<_>>()
            .join(", "),
          chunk.as_u32()
        )));
      }

      final_source.add(RawSource::from_static("\n"));
    }

    if !required_symbols.is_empty() {
      let mut already_imported_chunks: UkeySet<ChunkUkey> = imported_symbols
        .keys()
        .map(|module_id| Self::get_module_chunk(*module_id, compilation))
        .collect();
      let mut extra_imports = vec![];

      let required_str = RawStringSource::from(
        required_symbols
          .iter()
          .map(|(id, atom)| {
            let ref_chunk = Self::get_module_chunk(*id, &compilation);
            if &ref_chunk != chunk_ukey && !already_imported_chunks.contains(&ref_chunk) {
              already_imported_chunks.insert(ref_chunk);
              extra_imports.push(format!(
                "import \"__RSPACK_ESM_CHUNK_{}\";\n",
                ref_chunk.as_u32()
              ));
            }

            format!(
              "const {} = __webpack_require__({});\n",
              atom,
              serde_json::to_string(
                ChunkGraph::get_module_id(&compilation.module_ids_artifact, *id)
                  .unwrap()
                  .as_str()
              )
              .unwrap()
            )
          })
          .collect::<Vec<_>>()
          .join("\n"),
      );

      final_source.add(RawStringSource::from(extra_imports.join("\n")));
      final_source.add(RawSource::from_static("\n"));
      final_source.add(required_str);
    }

    let mut export_specifiers = vec![];

    if chunk.has_runtime(&compilation.chunk_group_by_ukey) {
      let bootstrap = render_bootstrap(chunk_ukey, compilation).await?;
      final_source.add(RawStringSource::from_static(
        "\nvar __webpack_modules__ = {};\n",
      ));
      final_source.add(RawStringSource::from(bootstrap.header.join("\n")));
      final_source.add(RawSource::from("\n"));
      final_source.add(RawStringSource::from(bootstrap.startup.join("\n")));
      final_source.add(render_runtime_modules(compilation, chunk_ukey).await?);
      final_source.add(RawSource::from("\n"));

      export_specifiers.push(RuntimeGlobals::REQUIRE.name());
    }
    final_source.add(render_source);

    for (id, exports) in &chunk_link.exports {
      let info = concatenated_modules_map
        .get(id)
        .expect("should have info")
        .as_concatenated();
      for symbol in exports {
        let local_symbol = info.internal_names.get(&symbol).unwrap();
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
