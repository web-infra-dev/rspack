use std::{
  sync::{Arc, LazyLock},
  time::{Duration, Instant},
};

use camino::{Utf8Path, Utf8PathBuf};
use regex::Regex;
use rspack_collections::IdentifierSet;
use rspack_core::{
  Compilation, CompilationOptimizeDependencies, CompilationParams, CompilationProcessAssets,
  CompilerCompilation, DependencyType, ExportsInfoArtifact, FactoryMeta, ModuleFactoryCreateData,
  ModuleType, NormalModuleFactoryBeforeResolve, NormalModuleFactoryParser, ParserAndGenerator,
  ParserOptions, Plugin, ResolveContext, ResolveOptionsWithDependencyType, ResolveResult,
  SideEffectsOptimizeArtifact,
  build_module_graph::BuildModuleGraphArtifact,
  module_declared_side_effect_free,
  rspack_sources::{BoxSource, ReplaceSource, SourceExt},
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  BoxJavascriptParserPlugin, parser_and_generator::JavaScriptParserAndGenerator,
};
use rustc_hash::FxHashMap as HashMap;

static RSTEST_FLAG_RE: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"\/\* RSTEST:(MOCK|UNMOCK|MOCKREQUIRE|HOISTED):([^:]+):(.*?):(HOIST_START|HOIST_END|PLACEHOLDER) \*\/")
    .expect("should initialize rstest flag regex")
});

use crate::{
  esm_import_dependency::{
    RstestESMImportSideEffectDependencyTemplate, RstestESMImportSpecifierDependencyTemplate,
  },
  import_dependency::ImportDependencyTemplate,
  mock_method_dependency::MockMethodDependencyTemplate,
  mock_module_id_dependency::{MockModuleIdDependency, MockModuleIdDependencyTemplate},
  module_path_name_dependency::ModulePathNameDependencyTemplate,
  parser_plugin::{MOCK_TARGET_REQUEST_PREFIX, RstestParserPlugin},
  url_dependency::RstestUrlDependencyTemplate,
};

#[derive(Debug)]
pub struct RstestPluginOptions {
  pub module_path_name: bool,
  pub hoist_mock_module: bool,
  pub import_meta_path_name: bool,
  pub manual_mock_root: String,
  pub preserve_new_url: Vec<String>,
  pub globals: bool,
}

#[derive(Debug)]
pub struct ProgressPluginStateInfo {
  pub value: String,
  pub time: Instant,
  pub duration: Option<Duration>,
}

#[plugin]
#[derive(Debug)]
pub struct RstestPlugin {
  options: RstestPluginOptions,
}

impl RstestPlugin {
  pub fn new(options: RstestPluginOptions) -> Self {
    Self::new_inner(options)
  }

  fn calc_default_mocked_target(&self, value: &str) -> Utf8PathBuf {
    let stripped = value.strip_prefix("node:").unwrap_or(value);
    let path_buf = Utf8PathBuf::from(stripped);

    if stripped.starts_with('.') {
      path_buf.parent().map_or_else(
        || Utf8PathBuf::from("__mocks__").join(&path_buf),
        |p| {
          p.join("__mocks__")
            .join(path_buf.file_name().unwrap_or_default())
        },
      )
    } else {
      Utf8PathBuf::from(&self.options.manual_mock_root).join(&path_buf)
    }
  }

  fn synthetic_mock_dep(data: &ModuleFactoryCreateData) -> bool {
    data.request.starts_with(MOCK_TARGET_REQUEST_PREFIX)
  }

  fn resolve_directory_mock_target(
    &self,
    request: &Utf8Path,
    context: &Utf8Path,
    resolved_path: &Utf8Path,
    main_files: impl Iterator<Item = String>,
  ) -> Option<Utf8PathBuf> {
    let requested_path =
      Utf8PathBuf::from_path_buf(context.join(request).as_std_path().to_path_buf()).ok()?;
    let resolved_parent = resolved_path.parent()?;
    if resolved_parent != requested_path {
      return None;
    }

    let resolved_stem = resolved_path.file_stem()?;
    main_files
      .into_iter()
      .find(|main_file| main_file == resolved_stem)
      .map(|main_file| request.join("__mocks__").join(main_file))
  }

  async fn resolve_mock_request(&self, data: &mut ModuleFactoryCreateData) {
    let Some(dep) = data.dependencies.first() else {
      return;
    };
    let dependency_category = *dep.category();
    let request = data
      .request
      .strip_prefix(MOCK_TARGET_REQUEST_PREFIX)
      .unwrap_or(&data.request)
      .to_string();
    let stripped = request.strip_prefix("node:").unwrap_or(&request);
    let default_target = self.calc_default_mocked_target(&request);

    if !stripped.starts_with('.') {
      let resolved_request = default_target.to_string();
      if let Some(dep) = data
        .dependencies
        .first_mut()
        .and_then(|dep| dep.downcast_mut::<MockModuleIdDependency>())
      {
        dep.set_request(resolved_request.clone());
      }
      data.request = resolved_request;
      return;
    }

    let dep = ResolveOptionsWithDependencyType {
      resolve_options: data
        .resolve_options
        .clone()
        .map(|options| Box::new(Arc::unwrap_or_clone(options))),
      resolve_to_context: false,
      dependency_category,
    };
    let resolver = data.resolver_factory.get(dep);
    let mut resolve_context = ResolveContext::default();

    let resolved_directory_target = match resolver
      .resolve_with_context(data.context.as_ref(), stripped, &mut resolve_context)
      .await
    {
      Ok(ResolveResult::Resource(resource)) => self.resolve_directory_mock_target(
        Utf8Path::new(stripped),
        data.context.as_ref(),
        &resource.path,
        resolver.options().main_files().cloned(),
      ),
      _ => None,
    };

    data.add_file_dependencies(resolve_context.file_dependencies);
    data.add_missing_dependencies(resolve_context.missing_dependencies);
    let resolved_request = resolved_directory_target
      .unwrap_or(default_target)
      .to_string();
    if let Some(dep) = data
      .dependencies
      .first_mut()
      .and_then(|dep| dep.downcast_mut::<MockModuleIdDependency>())
    {
      dep.set_request(resolved_request.clone());
    }
    data.request = resolved_request;
  }

  fn update_source(&self, old: BoxSource, replace_map: &HashMap<String, MockFlagPos>) -> BoxSource {
    let old_source = old.clone();
    let mut replace = ReplaceSource::new(old_source);

    for pos in replace_map.values() {
      if let (Some(placeholder_start), Some(placeholder_end)) =
        (pos.placeholder_start, pos.placeholder_end)
        && let (
          Some(content_start),
          Some(content_end),
          Some(content_with_flag_start),
          Some(content_with_flag_end),
        ) = (
          pos.content_start,
          pos.content_end,
          pos.content_with_flag_start,
          pos.content_with_flag_end,
        )
      {
        let content = &old.source().into_string_lossy()[content_start..content_end];
        replace.replace(
          placeholder_start as u32,
          placeholder_end as u32 + 1, // consider the trailing semicolon
          format! {"// [Rstest mock hoist] \"{}\"\n{content};\n\n", pos.request},
          None,
        );
        replace.replace_static(
          content_with_flag_start as u32,
          content_with_flag_end as u32,
          "",
          None,
        );
      }
    }

    replace.boxed()
  }
}

#[plugin_hook(NormalModuleFactoryBeforeResolve for RstestPlugin)]
async fn nmf_before_resolve(&self, data: &mut ModuleFactoryCreateData) -> Result<Option<bool>> {
  if Self::synthetic_mock_dep(data) {
    self.resolve_mock_request(data).await;
  }

  Ok(None)
}

#[plugin_hook(NormalModuleFactoryParser for RstestPlugin)]
async fn nmf_parser(
  &self,
  module_type: &ModuleType,
  parser: &mut Box<dyn ParserAndGenerator>,
  _parser_options: Option<&ParserOptions>,
) -> Result<()> {
  if module_type.is_js_like()
    && let Some(parser) = parser.downcast_mut::<JavaScriptParserAndGenerator>()
  {
    parser.add_parser_plugin(Box::new(RstestParserPlugin::new(
      crate::parser_plugin::RstestParserPluginOptions {
        module_path_name: self.options.module_path_name,
        hoist_mock_module: self.options.hoist_mock_module,
        import_meta_path_name: self.options.import_meta_path_name,
        manual_mock_root: self.options.manual_mock_root.clone(),
        globals: self.options.globals,
      },
    )) as BoxJavascriptParserPlugin);
  }

  Ok(())
}

#[plugin_hook(CompilerCompilation for RstestPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_template(
    ModulePathNameDependencyTemplate::template_type(),
    Arc::new(ModulePathNameDependencyTemplate::default()),
  );

  compilation.set_dependency_template(
    MockMethodDependencyTemplate::template_type(),
    Arc::new(MockMethodDependencyTemplate::default()),
  );

  compilation.set_dependency_template(
    MockModuleIdDependencyTemplate::template_type(),
    Arc::new(MockModuleIdDependencyTemplate::default()),
  );

  Ok(())
}

#[plugin_hook(CompilerCompilation for RstestPlugin, stage = 9999)]
async fn compilation_stage_9999(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  // Override ESM import template for importActual hoist ordering.
  compilation.set_dependency_template(
    RstestESMImportSideEffectDependencyTemplate::template_type(),
    Arc::new(RstestESMImportSideEffectDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    RstestESMImportSpecifierDependencyTemplate::template_type(),
    Arc::new(RstestESMImportSpecifierDependencyTemplate::default()),
  );

  // Override the default import dependency template.
  compilation.set_dependency_template(
    ImportDependencyTemplate::template_type(),
    Arc::new(ImportDependencyTemplate::default()),
  );

  if !self.options.preserve_new_url.is_empty() {
    compilation.set_dependency_template(
      RstestUrlDependencyTemplate::template_type(),
      Arc::new(RstestUrlDependencyTemplate::new(
        self.options.preserve_new_url.clone(),
      )),
    );
  }

  Ok(())
}

#[derive(Debug, Default)]
struct MockFlagPos {
  request: String,
  content_start: Option<usize>,
  content_with_flag_start: Option<usize>,
  content_end: Option<usize>,
  content_with_flag_end: Option<usize>,
  placeholder_start: Option<usize>,
  placeholder_end: Option<usize>,
}

#[plugin_hook(CompilationProcessAssets for RstestPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_ADDITIONAL)]
async fn mock_hoist_process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let mut files = Vec::with_capacity(compilation.build_chunk_graph_artifact.chunk_by_ukey.len());

  for chunk in compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .values()
  {
    for file in chunk.files() {
      files.push(file.clone());
    }
  }

  for file in files {
    let mut pos_map: HashMap<String, MockFlagPos> = HashMap::default();
    let _res = compilation.update_asset(file.as_str(), |old, info| {
      // Only handles JavaScript.
      if info.javascript_module.is_none() {
        return Ok((old, info));
      }

      let content = old.source().into_string_lossy();
      let captures: Vec<_> = RSTEST_FLAG_RE.captures_iter(&content).collect();

      for c in captures {
        let [Some(full), Some(hoist_id), Some(request), Some(t)] =
          [c.get(0), c.get(2), c.get(3), c.get(4)]
        else {
          continue;
        };

        let entry = pos_map.entry(hoist_id.as_str().to_string()).or_default();
        entry.request = request.as_str().to_string();

        if t.as_str() == "HOIST_START" {
          entry.content_with_flag_start = Some(full.start());
          entry.content_start = Some(full.end());
        } else if t.as_str() == "HOIST_END" {
          entry.content_with_flag_end = Some(full.end());
          entry.content_end = Some(full.start());
        } else if t.as_str() == "PLACEHOLDER" {
          entry.placeholder_start = Some(full.start());
          entry.placeholder_end = Some(full.end());
        } else {
          panic!(
            "Unknown rstest mock type: {}",
            c.get(1).map_or("", |m| m.as_str())
          );
        }
      }

      let new = self.update_source(old, &pos_map);
      Ok((new, info))
    });
  }

  Ok(())
}

#[plugin_hook(CompilationOptimizeDependencies for RstestPlugin, stage = -1000)]
async fn optimize_dependencies(
  &self,
  _compilation: &Compilation,
  _side_effects_optimize_artifact: &mut SideEffectsOptimizeArtifact,
  build_module_graph_artifact: &mut BuildModuleGraphArtifact,
  _exports_info_artifact: &mut ExportsInfoArtifact,
  _diagnostics: &mut Vec<Diagnostic>,
) -> Result<Option<bool>> {
  let mocked_module_ids: IdentifierSet = {
    let module_graph = build_module_graph_artifact.get_module_graph();
    module_graph
      .dependencies()
      .filter(|(_, dep)| dep.dependency_type() == &DependencyType::RstestMockModuleId)
      .filter_map(|(dep_id, _)| {
        module_graph
          .module_identifier_by_dependency_id(dep_id)
          .copied()
      })
      .collect()
  };

  let mut updated_mocked_module_ids = IdentifierSet::default();
  let module_graph = build_module_graph_artifact.get_module_graph_mut();
  for module_id in mocked_module_ids {
    if let Some(module) = module_graph.module_by_identifier_mut(&module_id)
      && module_declared_side_effect_free(module.as_ref()) == Some(true)
    {
      module.set_factory_meta(FactoryMeta {
        side_effect_free: Some(false),
      });
      updated_mocked_module_ids.insert(module_id);
    }
  }

  if updated_mocked_module_ids.is_empty() {
    return Ok(None);
  }

  build_module_graph_artifact
    .side_effects_state_artifact
    .write()
    .expect("should lock side effects state artifact")
    .bump_version();

  Ok(None)
}

impl Plugin for RstestPlugin {
  fn name(&self) -> &'static str {
    "rstest"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));

    ctx
      .normal_module_factory_hooks
      .before_resolve
      .tap(nmf_before_resolve::new(self));

    ctx
      .compiler_hooks
      .compilation
      .tap(compilation_stage_9999::new(self));

    if self.options.module_path_name {
      ctx
        .normal_module_factory_hooks
        .parser
        .tap(nmf_parser::new(self));
    }

    ctx
      .compilation_hooks
      .optimize_dependencies
      .tap(optimize_dependencies::new(self));

    if self.options.hoist_mock_module {
      ctx
        .compilation_hooks
        .process_assets
        .tap(mock_hoist_process_assets::new(self));
    }

    Ok(())
  }
}
