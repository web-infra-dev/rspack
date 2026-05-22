use std::{borrow::Cow, sync::LazyLock};

use cow_utils::CowUtils;
use regex::Regex;
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  Chunk, ChunkGroupUkey, ChunkUkey, Compilation, Module, ModuleGraph, ModuleIdentifier,
  get_undo_path,
  rspack_sources::{
    BoxSource, ConcatSource, RawStringSource, SourceExt, SourceMap, SourceMapSource,
    WithoutOriginalOptions,
  },
};
use rustc_hash::FxHashSet;

static MEDIA_RE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r#";|\s*$"#).expect("should compile"));
static STARTS_WITH_AT_IMPORT: &str = "@import url";

#[derive(Debug)]
pub struct CssExtractOrderConflict {
  pub chunk: ChunkUkey,
  pub fallback_module: ModuleIdentifier,
  pub reasons: Vec<(ModuleIdentifier, Option<String>, Option<String>)>,
}

pub struct CssExtractAssetModule<'a> {
  pub module: &'a dyn Module,
  pub content: &'a str,
  pub media: Option<&'a str>,
  pub supports: Option<&'a str>,
  pub source_map: Option<&'a str>,
  pub css_layer: Option<&'a str>,
}

pub struct CssExtractAssetRenderOptions<'a> {
  pub chunk: &'a Chunk,
  pub filename: &'a str,
  pub compilation: &'a Compilation,
  pub pathinfo: bool,
  pub enforce_relative: bool,
  pub base_uri: &'static str,
  pub absolute_public_path: &'static str,
  pub auto_public_path: &'static str,
  pub single_dot_path_segment: &'static str,
}

pub fn get_extract_modules_in_order<'comp>(
  chunk: &Chunk,
  modules: &[&dyn Module],
  compilation: &'comp Compilation,
  module_graph: &'comp ModuleGraph,
  ignore_order: bool,
) -> (Vec<&'comp dyn Module>, Option<Vec<CssExtractOrderConflict>>) {
  let mut module_deps_reasons: IdentifierMap<IdentifierMap<FxHashSet<ChunkGroupUkey>>> = modules
    .iter()
    .map(|m| (m.identifier(), Default::default()))
    .collect();

  let mut module_dependencies: IdentifierMap<IdentifierSet> = modules
    .iter()
    .map(|module| (module.identifier(), IdentifierSet::default()))
    .collect();

  let mut groups = chunk.groups().iter().copied().collect::<Vec<_>>();
  groups.sort_by(|a, b| {
    let a = compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .expect_get(a);
    let b = compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .expect_get(b);
    match a.index.cmp(&b.index) {
      std::cmp::Ordering::Equal => a.ukey.cmp(&b.ukey),
      order_res => order_res,
    }
  });

  let mut modules_by_chunk_group = groups
    .iter()
    .map(|chunk_group| {
      let chunk_group = compilation
        .build_chunk_graph_artifact
        .chunk_group_by_ukey
        .expect_get(chunk_group);
      let mut sorted_module = modules
        .iter()
        .map(|module| {
          let identifier = module.identifier();
          (identifier, chunk_group.module_post_order_index(&identifier))
        })
        .filter_map(|(id, idx)| idx.map(|idx| (id, idx)))
        .collect::<Vec<_>>();

      sorted_module.sort_by(|(_, idx1), (_, idx2)| idx2.cmp(idx1));

      for (i, (module, _)) in sorted_module.iter().enumerate() {
        let set = module_dependencies
          .get_mut(module)
          .expect("should have module before");

        let reasons = module_deps_reasons
          .get_mut(module)
          .expect("should have module dep reason");

        let mut j = i + 1;
        while j < sorted_module.len() {
          let (module, _) = sorted_module[j];
          set.insert(module);

          let reason = reasons.entry(module).or_default();
          reason.insert(chunk_group.ukey);

          j += 1;
        }
      }

      sorted_module
    })
    .collect::<Vec<Vec<(ModuleIdentifier, u32)>>>();

  let mut used_modules: IdentifierSet = Default::default();
  let mut result: Vec<&dyn Module> = Default::default();
  let mut conflicts: Option<Vec<CssExtractOrderConflict>> = None;

  while used_modules.len() < modules.len() {
    let mut success = false;
    let mut best_match: Option<Vec<ModuleIdentifier>> = None;
    let mut best_match_deps: Option<Vec<ModuleIdentifier>> = None;

    for list in &mut modules_by_chunk_group {
      while !list.is_empty()
        && used_modules.contains(&list.last().expect("should have list item").0)
      {
        list.pop();
      }

      if !list.is_empty() {
        let module = list.last().expect("should have item").0;
        let deps = module_dependencies.get(&module).expect("should have deps");
        let failed_deps = deps
          .iter()
          .filter(|dep| !used_modules.contains(dep))
          .copied()
          .collect::<Vec<_>>();

        let failed_count = failed_deps.len();

        if best_match_deps.is_none()
          || best_match_deps
            .as_ref()
            .expect("should have best match dep")
            .len()
            > failed_deps.len()
        {
          best_match = Some(list.iter().map(|(id, _)| *id).collect());
          best_match_deps = Some(failed_deps);
        }

        if failed_count == 0 {
          list.pop();
          used_modules.insert(module);
          result.push(
            module_graph
              .module_by_identifier(&module)
              .expect("should have module")
              .as_ref(),
          );
          success = true;
          break;
        }
      }
    }

    if !success {
      let mut best_match = best_match.expect("should have best match");
      let best_match_deps = best_match_deps.expect("should have best match");
      let fallback_module = best_match.pop().expect("should have best match");
      if !ignore_order {
        let reasons = module_deps_reasons
          .get(&fallback_module)
          .expect("should have dep reason");

        let new_conflict = CssExtractOrderConflict {
          chunk: chunk.ukey(),
          fallback_module,
          reasons: best_match_deps
            .into_iter()
            .map(|m| {
              let good_reasons_map = module_deps_reasons.get(&m);
              let good_reasons = good_reasons_map.and_then(|reasons| reasons.get(&fallback_module));

              let failed_chunk_groups = reasons.get(&m).map(|reasons| {
                reasons
                  .iter()
                  .filter_map(|cg| {
                    let chunk_group = compilation
                      .build_chunk_graph_artifact
                      .chunk_group_by_ukey
                      .expect_get(cg);

                    chunk_group.name()
                  })
                  .collect::<Vec<_>>()
                  .join(",")
              });

              let good_chunk_groups = good_reasons.map(|reasons| {
                reasons
                  .iter()
                  .filter_map(|cg| {
                    compilation
                      .build_chunk_graph_artifact
                      .chunk_group_by_ukey
                      .expect_get(cg)
                      .name()
                  })
                  .collect::<Vec<_>>()
                  .join(", ")
              });

              (m, failed_chunk_groups, good_chunk_groups)
            })
            .collect(),
        };
        if let Some(conflicts) = &mut conflicts {
          conflicts.push(new_conflict);
        } else {
          conflicts = Some(vec![new_conflict]);
        }
      }

      used_modules.insert(fallback_module);
      result.push(
        module_graph
          .module_by_identifier(&fallback_module)
          .expect("should have fallback module")
          .as_ref(),
      );
    }
  }

  (result, conflicts)
}

pub fn render_extract_css_asset(
  modules: &[CssExtractAssetModule<'_>],
  options: &CssExtractAssetRenderOptions<'_>,
) -> BoxSource {
  let mut source = ConcatSource::default();
  let mut external_source = ConcatSource::default();

  for module in modules {
    let content = Cow::Borrowed(module.content);
    let readable_identifier = module
      .module
      .readable_identifier(&options.compilation.options.context);
    let starts_with_at_import = content.starts_with(STARTS_WITH_AT_IMPORT);

    let header = options.pathinfo.then(|| {
      let req_str = readable_identifier.cow_replace("*/", "*_/");
      let req_str_star = "*".repeat(req_str.len());
      RawStringSource::from(format!(
        r#"/*!****{req_str_star}****!*\
  !*** {req_str} ***!
  \****{req_str_star}****/
"#
      ))
    });

    if starts_with_at_import {
      if let Some(header) = header {
        external_source.add(header);
      }
      if let Some(media) = module.media {
        let new_content = MEDIA_RE.replace_all(content.as_ref(), media);
        external_source.add(RawStringSource::from(new_content.to_string() + "\n"));
      } else {
        external_source.add(RawStringSource::from(content.to_string() + "\n"));
      }
    } else {
      let mut need_supports = false;
      let mut need_media = false;

      if let Some(header) = header {
        source.add(header);
      }

      if let Some(supports) = module.supports
        && !supports.is_empty()
      {
        need_supports = true;
        source.add(RawStringSource::from(format!(
          "@supports ({supports}) {{\n"
        )));
      }

      if let Some(media) = module.media
        && !media.is_empty()
      {
        need_media = true;
        source.add(RawStringSource::from(format!("@media {media} {{\n")));
      }

      if let Some(layer) = module.css_layer {
        source.add(RawStringSource::from(format!("@layer {layer} {{\n")));
      }

      let undo_path = get_undo_path(
        options.filename,
        options.compilation.options.output.path.to_string(),
        options.enforce_relative,
      );

      let content = content.cow_replace(options.absolute_public_path, "");
      let content = content.cow_replace(options.single_dot_path_segment, ".");
      let content = content.cow_replace(options.auto_public_path, &undo_path);
      let content = content.cow_replace(
        options.base_uri,
        options
          .chunk
          .get_entry_options(
            &options
              .compilation
              .build_chunk_graph_artifact
              .chunk_group_by_ukey,
          )
          .and_then(|entry_options| entry_options.base_uri.as_ref())
          .unwrap_or(&undo_path),
      );

      if let Some(source_map) = module.source_map {
        source.add(SourceMapSource::new(WithoutOriginalOptions {
          value: content.to_string(),
          name: readable_identifier,
          source_map: SourceMap::from_json(source_map).expect("invalid sourcemap"),
        }))
      } else {
        source.add(RawStringSource::from(content.to_string()));
      }

      source.add(RawStringSource::from_static("\n"));

      if need_media {
        source.add(RawStringSource::from_static("}\n"));
      }

      if need_supports {
        source.add(RawStringSource::from_static("}\n"));
      }

      if module.css_layer.is_some() {
        source.add(RawStringSource::from_static("}\n"));
      }
    }
  }

  external_source.add(source);
  external_source.boxed()
}
