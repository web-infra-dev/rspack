use crate::generate_rspack_execute;
use crate::utils::parse_file;
use crate::visitors::ClearMark;
use crate::{module::JsModule, utils::get_swc_compiler};
use anyhow::{Context, Result};
use rayon::prelude::*;
use rspack_core::{
  AssetContent, Filename, ModuleAst, ModuleRenderResult, ModuleType, OutputFilename,
  ParseModuleArgs, Parser, Plugin, PluginContext, PluginRenderManifestHookOutput,
  RenderManifestEntry, SourceType,
};

use swc_common::comments::SingleThreadedComments;
use swc_common::Mark;
use swc_ecma_transforms::react::{react, Options as ReactOptions};
use swc_ecma_transforms::{react as swc_react, resolver};
use swc_ecma_visit::{as_folder, FoldWith};
use tracing::instrument;

#[derive(Debug)]
pub struct JsPlugin {}

impl Plugin for JsPlugin {
  fn name(&self) -> &'static str {
    "javascript"
  }
  fn apply(&mut self, ctx: PluginContext<&mut rspack_core::ApplyContext>) -> anyhow::Result<()> {
    ctx
      .context
      .register_parser(ModuleType::Js, Box::new(JsParser {}));
    ctx
      .context
      .register_parser(ModuleType::Ts, Box::new(JsParser {}));
    ctx
      .context
      .register_parser(ModuleType::Tsx, Box::new(JsParser {}));
    ctx
      .context
      .register_parser(ModuleType::Jsx, Box::new(JsParser {}));

    Ok(())
  }

  #[instrument(skip_all)]
  fn render_manifest(
    &self,
    _ctx: PluginContext,
    args: rspack_core::RenderManifestArgs,
  ) -> PluginRenderManifestHookOutput {
    let compilation = args.compilation;
    let module_graph = &compilation.module_graph;
    let namespace = &compilation.options.output.namespace;
    let chunk = compilation
      .chunk_graph
      .chunk_by_id(args.chunk_id)
      .ok_or_else(|| anyhow::format_err!("Not found chunk {:?}", args.chunk_id))?;
    let ordered_modules = chunk.ordered_modules(module_graph);

    let code = ordered_modules
      .par_iter()
      .filter(|module| {
        module
          .module
          .source_types(module, compilation)
          .contains(&SourceType::JavaScript)
      })
      .map(|module| {
        module
          .module
          .render(SourceType::JavaScript, module, compilation)
          .map(|source| {
            if let Some(ModuleRenderResult::JavaScript(source)) = source {
              Some(source)
            } else {
              None
            }
          })
      })
      .collect::<Result<Vec<Option<String>>>>()?
      .into_par_iter()
      .flatten()
      .chain([{
        if chunk.kind.is_entry() {
          generate_rspack_execute(
            namespace,
            ordered_modules
              .last()
              .ok_or_else(|| anyhow::format_err!("TODO:"))?
              .id
              .as_str(),
          )
        } else {
          String::new()
        }
      }])
      .fold(String::new, |mut output, cur| {
        output += &cur;
        output
      })
      .collect::<String>();

    Ok(vec![RenderManifestEntry::new(
      AssetContent::String(code),
      OutputFilename::new("[name][ext]".to_owned())
        .filename(args.chunk_id.to_owned(), ".js".to_owned()),
    )])
  }
}

#[derive(Debug)]
struct JsParser {}

impl Parser for JsParser {
  fn parse(
    &self,
    module_type: ModuleType,
    args: ParseModuleArgs,
  ) -> anyhow::Result<rspack_core::BoxModule> {
    if !module_type.is_js_like() {
      anyhow::bail!(
        "`module_type` {:?} not supported for `JsParser`",
        module_type
      );
    }
    let ast = {
      match args.ast {
        Some(ModuleAst::JavaScript(_ast)) => Ok::<_, anyhow::Error>(_ast),
        None => {
          if let Some(content) = args.source {
            Ok(parse_file(
              content
                .try_into_string()
                .context("Unable to serialize content as string which is required by plugin css")?,
              args.uri,
              &module_type,
            ))
          } else {
            anyhow::bail!(
              "ast and source is both empty for {}, or content type does not match {:?}",
              args.uri,
              args.source
            )
          }
        }
        _ => anyhow::bail!("not supported ast {:?} for js parser", args.ast),
      }
    }?;

    let ast = get_swc_compiler().run(|| {
      let top_level_mark = Mark::new();
      let mut react_folder = react::<SingleThreadedComments>(
        get_swc_compiler().cm.clone(),
        None,
        ReactOptions {
          development: Some(false),
          runtime: Some(swc_react::Runtime::Automatic),
          refresh: None,
          ..Default::default()
        },
        Mark::new(),
      );

      let ast = ast.fold_with(&mut resolver(Mark::new(), top_level_mark, false));
      let ast = ast.fold_with(&mut react_folder);
      ast.fold_with(&mut as_folder(ClearMark))
    });
    Ok(Box::new(JsModule {
      ast,
      uri: args.uri.to_string(),
      module_type,
    }))
  }
}
