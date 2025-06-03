use std::{
  sync::Arc,
  time::{Duration, Instant},
};

use async_trait::async_trait;
use rspack_core::{
  rspack_sources::{BoxSource, ReplaceSource, SourceExt},
  ApplyContext, Compilation, CompilationParams, CompilationProcessAssets, CompilerCompilation,
  CompilerOptions, ModuleType, NormalModuleFactoryParser, ParserAndGenerator, ParserOptions,
  Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  parser_and_generator::JavaScriptParserAndGenerator, BoxJavascriptParserPlugin,
};

use crate::{
  mock_hoist_dependency::MockHoistDependencyTemplate,
  mock_module_id_dependency::MockModuleIdDependencyTemplate,
  module_path_name_dependency::ModulePathNameDependencyTemplate, parser_plugin::RstestParserPlugin,
};

#[derive(Debug)]
pub struct RstestPluginOptions {
  pub module_path_name: bool,
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
    let old_source = old.to_owned();
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
        let content = old.source()[content_start..content_end].to_string();
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
  parser: &mut dyn ParserAndGenerator,
  _parser_options: Option<&ParserOptions>,
) -> Result<()> {
  if module_type.is_js_like()
    && let Some(parser) = parser.downcast_mut::<JavaScriptParserAndGenerator>()
  {
    parser.add_parser_plugin(Box::<RstestParserPlugin>::default() as BoxJavascriptParserPlugin);
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
    MockHoistDependencyTemplate::template_type(),
    Arc::new(MockHoistDependencyTemplate::default()),
  );

  compilation.set_dependency_template(
    MockModuleIdDependencyTemplate::template_type(),
    Arc::new(MockModuleIdDependencyTemplate::default()),
  );
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
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let mut files = vec![];

  for chunk in compilation.chunk_by_ukey.values() {
    for file in chunk.files() {
      files.push(file.clone());
    }
  }

  let regex =
    regex::Regex::new(r"\/\* RSTEST:MOCK_(.*?):(.*?) \*\/").expect("should initialize `Regex`");
  let mut pos_map: std::collections::HashMap<String, MockFlagPos> =
    std::collections::HashMap::new();

  for file in files {
    let _res = compilation.update_asset(file.as_str(), |old, info| {
      let content = old.source().to_string();
      let captures: Vec<_> = regex.captures_iter(&content).collect();

      for c in captures {
        let [Some(full), Some(t), Some(request)] = [c.get(0), c.get(1), c.get(2)] else {
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

#[async_trait]
impl Plugin for RstestPlugin {
  fn name(&self) -> &'static str {
    "rstest"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    if self.options.module_path_name {
      ctx
        .context
        .compiler_hooks
        .compilation
        .tap(compilation::new(self));

      ctx
        .context
        .normal_module_factory_hooks
        .parser
        .tap(nmf_parser::new(self));

      ctx
        .context
        .compilation_hooks
        .process_assets
        .tap(process_assets::new(self));
    }

    Ok(())
  }
}
