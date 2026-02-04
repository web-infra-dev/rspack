use std::{
  sync::Arc,
  time::{Duration, Instant},
};

use rspack_core::{
  Compilation, CompilationParams, CompilationProcessAssets, CompilerCompilation, ModuleType,
  NormalModuleFactoryParser, ParserAndGenerator, ParserOptions, Plugin,
  rspack_sources::{BoxSource, ReplaceSource, SourceExt},
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  BoxJavascriptParserPlugin, parser_and_generator::JavaScriptParserAndGenerator,
};

use crate::{
  import_dependency::ImportDependencyTemplate,
  mock_method_dependency::MockMethodDependencyTemplate,
  mock_module_id_dependency::MockModuleIdDependencyTemplate,
  module_path_name_dependency::ModulePathNameDependencyTemplate, parser_plugin::RstestParserPlugin,
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

  fn update_source(
    &self,
    old: BoxSource,
    replace_map: &std::collections::HashMap<String, MockFlagPos>,
  ) -> BoxSource {
    let old_source = old.clone();
    let mut replace = ReplaceSource::new(old_source);

    for (mocked_id, pos) in replace_map {
      if let (
        Some(content_start),
        Some(content_end),
        Some(placeholder_start),
        Some(placeholder_end),
        Some(content_with_flag_start),
        Some(content_with_flag_end),
      ) = (
        pos.content_start,
        pos.content_end,
        pos.placeholder_start,
        pos.placeholder_end,
        pos.content_with_flag_start,
        pos.content_with_flag_end,
      ) {
        let content = &old.source().into_string_lossy()[content_start..content_end];
        replace.replace(
          placeholder_start as u32,
          placeholder_end as u32 + 1, // consider the trailing semicolon
          &format! {"// [Rstest mock hoist] \"{mocked_id}\"\n{content};\n\n"},
          None,
        );
        replace.replace(
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

#[derive(Debug)]
struct MockFlagPos {
  content_start: Option<usize>,
  content_with_flag_start: Option<usize>,
  content_end: Option<usize>,
  content_with_flag_end: Option<usize>,
  placeholder_start: Option<usize>,
  placeholder_end: Option<usize>,
}

#[plugin_hook(CompilationProcessAssets for RstestPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_ADDITIONAL)]
async fn mock_hoist_process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let mut files = Vec::with_capacity(compilation.chunk_by_ukey.len());

  for chunk in compilation.chunk_by_ukey.values() {
    for file in chunk.files() {
      files.push(file.clone());
    }
  }

  let regex = regex::Regex::new(r"\/\* RSTEST:(MOCK|UNMOCK|MOCKREQUIRE|HOISTED)_(.*?):(.*?) \*\/")
    .expect("should initialize `Regex`");

  for file in files {
    let mut pos_map: std::collections::HashMap<String, MockFlagPos> =
      std::collections::HashMap::new();
    let _res = compilation.update_asset(file.as_str(), |old, info| {
      // Only handles JavaScript.
      if info.javascript_module.is_none() {
        return Ok((old, info));
      }

      let content = old.source().into_string_lossy();
      let captures: Vec<_> = regex.captures_iter(&content).collect();

      for c in captures {
        let [Some(full), Some(t), Some(request)] = [c.get(0), c.get(2), c.get(3)] else {
          continue;
        };

        let entry = pos_map
          .entry(request.as_str().to_string())
          .or_insert_with(|| MockFlagPos {
            content_start: None,
            content_end: None,
            content_with_flag_start: None,
            content_with_flag_end: None,
            placeholder_start: None,
            placeholder_end: None,
          });

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

impl Plugin for RstestPlugin {
  fn name(&self) -> &'static str {
    "rstest"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));

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

    if self.options.hoist_mock_module {
      ctx
        .compilation_hooks
        .process_assets
        .tap(mock_hoist_process_assets::new(self));
    }

    Ok(())
  }
}
