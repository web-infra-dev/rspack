use std::{
  borrow::Cow,
  cmp::max,
  hash::Hash,
  sync::{atomic::AtomicBool, Arc},
};

use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  rspack_sources::{ConcatSource, RawSource, SourceMap, SourceMapSource, WithoutOriginalOptions},
  ApplyContext, AssetInfo, Chunk, ChunkGroupUkey, ChunkKind, ChunkUkey, Compilation,
  CompilationContentHash, CompilationParams, CompilationRenderManifest,
  CompilationRuntimeRequirementInTree, CompilerCompilation, CompilerOptions, Filename, Module,
  ModuleGraph, ModuleIdentifier, ModuleType, PathData, Plugin, PluginContext, RenderManifestEntry,
  RuntimeGlobals, SourceType,
};
use rspack_error::{Diagnostic, Result};
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_runtime::GetChunkFilenameRuntimeModule;
use rustc_hash::{FxHashMap, FxHashSet};
use ustr::Ustr;

use crate::{
  css_module::{CssModule, CssModuleFactory, DEPENDENCY_TYPE},
  parser_and_generator::CssExtractParserAndGenerator,
  runtime::CssLoadingRuntimeModule,
};
pub static PLUGIN_NAME: &str = "css-extract-rspack-plugin";

pub static MODULE_TYPE_STR: Lazy<Ustr> = Lazy::new(|| Ustr::from("css/mini-extract"));
pub static MODULE_TYPE: Lazy<ModuleType> = Lazy::new(|| ModuleType::Custom(*MODULE_TYPE_STR));
pub static SOURCE_TYPE: Lazy<[SourceType; 1]> =
  Lazy::new(|| [SourceType::Custom(*MODULE_TYPE_STR)]);

pub static AUTO_PUBLIC_PATH: &str = "__mini_css_extract_plugin_public_path_auto__";
pub static AUTO_PUBLIC_PATH_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(AUTO_PUBLIC_PATH).expect("should compile"));

pub static ABSOLUTE_PUBLIC_PATH: &str = "webpack:///mini-css-extract-plugin/";
pub static ABSOLUTE_PUBLIC_PATH_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(ABSOLUTE_PUBLIC_PATH).expect("should compile"));

pub static BASE_URI: &str = "webpack://";
pub static BASE_URI_RE: Lazy<Regex> = Lazy::new(|| Regex::new(BASE_URI).expect("should compile"));

pub static SINGLE_DOT_PATH_SEGMENT: &str = "__mini_css_extract_plugin_single_dot_path_segment__";
pub static SINGLE_DOT_PATH_SEGMENT_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(SINGLE_DOT_PATH_SEGMENT).expect("should compile"));

static STARTS_WITH_AT_IMPORT_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new("^@import url").expect("should compile"));

struct CssOrderConflicts {
  chunk: ChunkUkey,
  fallback_module: ModuleIdentifier,

  // (module, failed chunkGroups, fulfilled chunkGroups)
  reasons: Vec<(ModuleIdentifier, Option<String>, Option<String>)>,
}

#[plugin]
#[derive(Debug)]
pub struct PluginCssExtract {
  pub(crate) options: Arc<CssExtractOptions>,
  registered: AtomicBool,
}

impl Eq for PluginCssExtractInner {}

impl PartialEq for PluginCssExtractInner {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.options, &other.options)
  }
}

#[derive(Debug)]
pub struct CssExtractOptions {
  pub filename: Filename,
  pub chunk_filename: Filename,
  pub ignore_order: bool,
  pub insert: InsertType,
  pub attributes: FxHashMap<String, String>,
  pub link_type: Option<String>,
  pub runtime: bool,
  pub pathinfo: bool,
}

// impl PartialEq for CssExtractOptions {
//   fn eq(&self, other: &Self) -> bool {
//     let equal = self.ignore_order == other.ignore_order
//       && self.insert == other.insert
//       && self.attributes == other.attributes
//       && self.link_type == other.link_type
//       && self.runtime == other.runtime
//       && self.pathinfo == other.pathinfo;

//     if !equal {
//       return false;
//     }

//     // TODO: function eq
//     match (self.filename.template(), self.chunk_filename.template()) {
//       (None, None) => return true,
//       (None, Some(_)) => return false,
//       (Some(_), None) => return false,
//       (Some(a), Some(b)) => a == b,
//     }
//   }
// }

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InsertType {
  Fn(String),
  Selector(String),
  Default,
}

impl PluginCssExtract {
  pub fn new(options: CssExtractOptions) -> Self {
    Self::new_inner(Arc::new(options), false.into())
  }

  // port from https://github.com/webpack-contrib/mini-css-extract-plugin/blob/d5e540baf8280442e523530ebbbe31c57a4c4336/src/index.js#L1127
  fn sort_modules<'comp>(
    &self,
    chunk: &Chunk,
    modules: Vec<&dyn Module>,
    compilation: &'comp Compilation,
    module_graph: &'comp ModuleGraph<'comp>,
  ) -> (Vec<&'comp dyn Module>, Option<Vec<CssOrderConflicts>>) {
    let mut module_deps_reasons: FxHashMap<
      ModuleIdentifier,
      FxHashMap<ModuleIdentifier, FxHashSet<ChunkGroupUkey>>,
    > = modules
      .iter()
      .map(|m| (m.identifier(), Default::default()))
      .collect();

    let mut module_dependencies: FxHashMap<ModuleIdentifier, FxHashSet<ModuleIdentifier>> = modules
      .iter()
      .map(|module| (module.identifier(), FxHashSet::default()))
      .collect();

    let mut groups = chunk.groups.iter().cloned().collect::<Vec<_>>();
    groups.sort_unstable();

    let mut modules_by_chunk_group = groups
      .iter()
      .map(|chunk_group| {
        let chunk_group = compilation.chunk_group_by_ukey.expect_get(chunk_group);
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
      .collect::<Vec<Vec<(ModuleIdentifier, usize)>>>();

    let mut used_modules: FxHashSet<ModuleIdentifier> = Default::default();
    let mut result: Vec<&dyn Module> = Default::default();
    let mut conflicts: Option<Vec<CssOrderConflicts>> = None;

    while used_modules.len() < modules.len() {
      let mut success = false;
      let mut best_match: Option<Vec<ModuleIdentifier>> = None;
      let mut best_match_deps: Option<Vec<ModuleIdentifier>> = None;

      for list in &mut modules_by_chunk_group {
        // skip and remove already added modules
        while !list.is_empty()
          && used_modules.contains(&list.last().expect("should have list item").0)
        {
          list.pop();
        }

        // skip empty lists
        if !list.is_empty() {
          let module = list.last().expect("should have item").0;
          let deps = module_dependencies.get(&module).expect("should have deps");
          let failed_deps = deps
            .iter()
            .filter(|dep| !used_modules.contains(dep))
            .cloned()
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
        // no module found => there is a conflict
        // use list with fewest failed deps
        // and emit a warning
        let mut best_match = best_match.expect("should have best match");
        let best_match_deps = best_match_deps.expect("should have best match");
        let fallback_module = best_match.pop().expect("should have best match");
        if !self.options.ignore_order {
          let reasons = module_deps_reasons
            .get(&fallback_module)
            .expect("should have dep reason");

          let new_conflict = CssOrderConflicts {
            chunk: chunk.ukey,
            fallback_module,
            reasons: best_match_deps
              .into_iter()
              .map(|m| {
                let good_reasons_map = module_deps_reasons.get(&m);
                let good_reasons =
                  good_reasons_map.and_then(|reasons| reasons.get(&fallback_module));

                let failed_chunk_groups = reasons.get(&m).map(|reasons| {
                  reasons
                    .iter()
                    .filter_map(|cg| {
                      let chunk_group = compilation.chunk_group_by_ukey.expect_get(cg);

                      chunk_group.name()
                    })
                    .collect::<Vec<_>>()
                    .join(",")
                });

                let good_chunk_groups = good_reasons.map(|reasons| {
                  reasons
                    .iter()
                    .filter_map(|cg| compilation.chunk_group_by_ukey.expect_get(cg).name())
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

  async fn render_content_asset<'comp>(
    &self,
    chunk: &Chunk,
    rendered_modules: Vec<&dyn Module>,
    filename_template: &Filename,
    compilation: &'comp Compilation,
    path_data: PathData<'comp>,
  ) -> Result<(RenderManifestEntry, Option<Vec<CssOrderConflicts>>)> {
    let module_graph = compilation.get_module_graph();
    // mini-extract-plugin has different conflict order in some cases,
    // for compatibility, we cannot use experiments.css sorting algorithm
    let (used_modules, conflicts) =
      self.sort_modules(chunk, rendered_modules, compilation, &module_graph);

    let used_modules = used_modules
      .into_iter()
      .filter_map(|module| module.downcast_ref::<CssModule>());

    let mut source = ConcatSource::default();
    let mut external_source = ConcatSource::default();

    let (filename, _) = compilation.get_path_with_info(filename_template, path_data)?;

    for module in used_modules {
      let content = Cow::Borrowed(module.content.as_str());
      let readable_identifier = module.readable_identifier(&compilation.options.context);
      let starts_with_at_import = STARTS_WITH_AT_IMPORT_REGEX.is_match(&content);

      let header = self.options.pathinfo.then(|| {
        let req_str = readable_identifier.replace("*/", "*_/");
        let req_str_star = "*".repeat(req_str.len());
        RawSource::from(format!(
          "/*!****{req_str_star}****!*\\\n  !*** {req_str} ***!\n  \\****{req_str_star}****/\n"
        ))
      });

      if starts_with_at_import {
        if let Some(header) = header {
          external_source.add(header);
        }
        if !module.media.is_empty() {
          static MEDIA_RE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r#";|\s*$"#).expect("should compile"));
          let new_content = MEDIA_RE.replace_all(content.as_ref(), &module.media);
          external_source.add(RawSource::from(new_content.to_string() + "\n"));
        } else {
          external_source.add(RawSource::from(content.to_string() + "\n"));
        }
      } else {
        if let Some(header) = header {
          source.add(header);
        }
        if !module.supports.is_empty() {
          source.add(RawSource::from(format!(
            "@supports ({}) {{\n",
            &module.supports
          )));
        }

        if !module.media.is_empty() {
          source.add(RawSource::from(format!("@media {} {{\n", &module.media)));
        }

        // TODO: layer support

        let undo_path = get_undo_path(
          &filename,
          compilation
            .options
            .output
            .path
            .to_str()
            .expect("should have output.path"),
          false,
        );

        let content = ABSOLUTE_PUBLIC_PATH_RE.replace_all(&content, "");
        let content = SINGLE_DOT_PATH_SEGMENT_RE.replace_all(&content, ".");
        let content = AUTO_PUBLIC_PATH_RE.replace_all(&content, &undo_path);
        let content = BASE_URI_RE.replace_all(
          &content,
          chunk
            .get_entry_options(&compilation.chunk_group_by_ukey)
            .and_then(|entry_options| entry_options.base_uri.as_ref())
            .unwrap_or(&undo_path),
        );

        if !module.source_map.is_empty() {
          source.add(SourceMapSource::new(WithoutOriginalOptions {
            value: content.to_string(),
            name: readable_identifier,
            source_map: SourceMap::from_json(&module.source_map).expect("invalid sourcemap"),
          }))
        } else {
          source.add(RawSource::from(content.to_string()));
        }

        source.add(RawSource::from("\n"));
        if !module.media.is_empty() {
          source.add(RawSource::from("}\n"));
        }
        if !module.supports.is_empty() {
          source.add(RawSource::from("}\n"));
        }
      }
    }

    external_source.add(source);
    Ok((
      RenderManifestEntry::new(
        Arc::new(external_source),
        filename,
        AssetInfo::default(),
        false,
        false,
      ),
      conflicts,
    ))
  }
}

#[plugin_hook(CompilerCompilation for PluginCssExtract)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(DEPENDENCY_TYPE.clone(), Arc::new(CssModuleFactory));

  if !self
    .registered
    .swap(true, std::sync::atomic::Ordering::Relaxed)
  {
    let (_, parser_and_generator) = compilation
      .plugin_driver
      .registered_parser_and_generator_builder
      .remove(&ModuleType::JsAuto)
      .expect("No JavaScript parser registered");

    compilation
      .plugin_driver
      .registered_parser_and_generator_builder
      .insert(
        ModuleType::JsAuto,
        Box::new(move |parser_opt, generator_opt| {
          let parser = parser_and_generator(parser_opt, generator_opt);
          Box::new(CssExtractParserAndGenerator::new(parser))
        }),
      );
  }
  Ok(())
}

#[plugin_hook(CompilationRuntimeRequirementInTree for PluginCssExtract)]
fn runtime_requirements_in_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
) -> Result<Option<()>> {
  if !self.options.runtime {
    return Ok(None);
  }

  let with_loading = runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS) && {
    let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);

    chunk
      .get_all_async_chunks(&compilation.chunk_group_by_ukey)
      .iter()
      .any(|chunk| {
        !compilation
          .chunk_graph
          .get_chunk_modules_by_source_type(chunk, SOURCE_TYPE[0], &compilation.get_module_graph())
          .is_empty()
      })
  };

  let with_hmr = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);

  if with_loading || with_hmr {
    if let Some(chunk_filename) = self.options.chunk_filename.template()
      && chunk_filename.contains("hash")
    {
      runtime_requirements_mut.insert(RuntimeGlobals::GET_FULL_HASH);
    }

    runtime_requirements_mut.insert(RuntimeGlobals::PUBLIC_PATH);

    let filename = self.options.filename.clone();
    let chunk_filename = self.options.chunk_filename.clone();

    compilation.add_runtime_module(
      chunk_ukey,
      Box::new(GetChunkFilenameRuntimeModule::new(
        "css",
        "mini-css",
        SOURCE_TYPE[0],
        "__webpack_require__.miniCssF".into(),
        |_| false,
        move |chunk, compilation| {
          chunk.content_hash.contains_key(&SOURCE_TYPE[0]).then(|| {
            if chunk.can_be_initial(&compilation.chunk_group_by_ukey) {
              filename.clone()
            } else {
              chunk_filename.clone()
            }
          })
        },
      )),
    )?;

    compilation.add_runtime_module(
      chunk_ukey,
      Box::new(CssLoadingRuntimeModule::new(
        *chunk_ukey,
        self.options.attributes.clone(),
        self.options.link_type.clone(),
        self.options.insert.clone(),
        with_loading,
        with_hmr,
      )),
    )?;
  }

  Ok(None)
}

#[plugin_hook(CompilationContentHash for PluginCssExtract)]
async fn content_hash(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  hashes: &mut FxHashMap<SourceType, RspackHash>,
) -> Result<()> {
  let module_graph = compilation.get_module_graph();

  let rendered_modules = compilation
    .chunk_graph
    .get_chunk_modules_iterable_by_source_type(chunk_ukey, SOURCE_TYPE[0], &module_graph)
    .collect::<Vec<_>>();

  if rendered_modules.is_empty() {
    return Ok(());
  }
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);

  let used_modules =
    rspack_plugin_css::CssPlugin::get_modules_in_order(chunk, rendered_modules, compilation)
      .0
      .into_iter()
      .filter_map(|module| module.downcast_ref::<CssModule>());

  let mut hasher = hashes
    .entry(SOURCE_TYPE[0])
    .or_insert_with(|| RspackHash::from(&compilation.options.output));

  used_modules
    .map(|m| {
      m.build_info()
        .expect("css module built")
        .hash
        .as_ref()
        .expect("css module should have hash")
    })
    .for_each(|current| {
      current.hash(&mut hasher);
    });

  Ok(())
}

#[plugin_hook(CompilationRenderManifest for PluginCssExtract)]
async fn render_manifest(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  manifest: &mut Vec<RenderManifestEntry>,
  diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
  let module_graph = compilation.get_module_graph();
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);

  if matches!(chunk.kind, ChunkKind::HotUpdate) {
    return Ok(());
  }

  let rendered_modules = compilation
    .chunk_graph
    .get_chunk_modules_iterable_by_source_type(chunk_ukey, SOURCE_TYPE[0], &module_graph)
    .collect::<Vec<_>>();

  if rendered_modules.is_empty() {
    return Ok(());
  }

  let filename_template = if chunk.can_be_initial(&compilation.chunk_group_by_ukey) {
    &self.options.filename
  } else {
    &self.options.chunk_filename
  };

  let (render_result, conflicts) = self
    .render_content_asset(
      chunk,
      rendered_modules,
      filename_template,
      compilation,
      PathData::default().chunk(chunk).content_hash_optional(
        chunk
          .content_hash
          .get(&SOURCE_TYPE[0])
          .map(|hash| hash.encoded()),
      ),
    )
    .await?;

  if let Some(conflicts) = conflicts {
    diagnostics.extend(conflicts.into_iter().map(|conflict| {
      let chunk = compilation.chunk_by_ukey.expect_get(&conflict.chunk);
      let fallback_module = module_graph
        .module_by_identifier(&conflict.fallback_module)
        .expect("should have module");

      Diagnostic::warn(
        "".into(),
        format!(
          "chunk {} [{PLUGIN_NAME}]\nConflicting order. Following module has been added:\n * {}
despite it was not able to fulfill desired ordering with these modules:\n{}",
          chunk
            .name
            .as_deref()
            .unwrap_or(chunk.id.as_deref().unwrap_or_default()),
          fallback_module.readable_identifier(&compilation.options.context),
          conflict
            .reasons
            .iter()
            .map(|(m, failed_reasons, good_reasons)| {
              let m = module_graph
                .module_by_identifier(m)
                .expect("should have module");

              format!(
                " * {}\n  - couldn't fulfill desired order of chunk group(s) {}{}",
                m.readable_identifier(&compilation.options.context),
                failed_reasons
                  .as_ref()
                  .map(|s| s.as_str())
                  .unwrap_or_default(),
                good_reasons
                  .as_ref()
                  .map(|s| format!(
                    "\n  - while fulfilling desired order of chunk group(s) {}",
                    s.as_str()
                  ))
                  .unwrap_or_default(),
              )
            })
            .collect::<Vec<_>>()
            .join("\n")
        ),
      )
    }));
  }
  manifest.push(render_result);

  Ok(())
}

#[async_trait::async_trait]
impl Plugin for PluginCssExtract {
  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));
    ctx
      .context
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirements_in_tree::new(self));
    ctx
      .context
      .compilation_hooks
      .content_hash
      .tap(content_hash::new(self));
    ctx
      .context
      .compilation_hooks
      .render_manifest
      .tap(render_manifest::new(self));

    Ok(())
  }
}

#[allow(clippy::unwrap_used)]
fn get_undo_path(filename: &str, output_path: &str, enforce_relative: bool) -> String {
  let mut depth: isize = -1;
  let mut append = "".into();

  // eslint-disable-next-line no-param-reassign
  let output_path = output_path.strip_suffix('\\').unwrap_or(output_path);
  let mut output_path = output_path
    .strip_suffix('/')
    .unwrap_or(output_path)
    .to_string();

  static PATH_SEP: Lazy<Regex> = Lazy::new(|| Regex::new(r#"[\\/]+"#).expect("should compile"));

  for part in PATH_SEP.split(filename) {
    if part == ".." {
      if depth > -1 {
        depth -= 1;
      } else {
        let i = output_path.find('/');
        let j = output_path.find('\\');
        let pos = if i.is_none() {
          j
        } else if j.is_none() {
          i
        } else {
          max(i, j)
        };

        if pos.is_none() {
          return format!("{output_path}/");
        }

        append = format!("{}/{append}", &output_path[pos.unwrap() + 1..]);

        output_path = output_path[0..pos.unwrap()].to_string();
      }
    } else if part != "." {
      depth += 1;
    }
  }

  if depth > 0 {
    format!("{}{append}", "../".repeat(depth as usize))
  } else if enforce_relative {
    format!("./{append}")
  } else {
    append
  }
}
