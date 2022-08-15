use crate::module::JS_MODULE_SOURCE_TYPE_LIST;
use crate::utils::{get_wrap_chunk_after, get_wrap_chunk_before, parse_file, wrap_module_function};
use crate::visitors::ClearMark;
use crate::{generate_rspack_execute, RSPACK_REGISTER};
use crate::{module::JsModule, utils::get_swc_compiler};
use anyhow::{Context, Result};
use rayon::prelude::*;
use rspack_core::{
  AssetContent, ChunkKind, FilenameRenderOptions, ModuleAst, ModuleRenderResult, ModuleType,
  ParseModuleArgs, Parser, Plugin, PluginContext, PluginRenderManifestHookOutput,
  RenderManifestEntry, SourceType,
};

use swc_common::comments::SingleThreadedComments;
use swc_common::Mark;
use swc_ecma_transforms::react::{react, Options as ReactOptions};
use swc_ecma_transforms::{react as swc_react, resolver};
use swc_ecma_visit::{as_folder, FoldWith};

#[derive(Debug)]
pub struct JsPlugin {
  unresolved_mark: Mark,
}

impl JsPlugin {
  pub fn new() -> Self {
    Self {
      unresolved_mark: get_swc_compiler().run(Mark::new),
    }
  }
}

impl Default for JsPlugin {
  fn default() -> Self {
    Self::new()
  }
}

impl Plugin for JsPlugin {
  fn name(&self) -> &'static str {
    "javascript"
  }
  fn apply(&mut self, ctx: PluginContext<&mut rspack_core::ApplyContext>) -> anyhow::Result<()> {
    ctx.context.register_parser(
      ModuleType::Js,
      Box::new(JsParser::new(self.unresolved_mark)),
    );
    ctx.context.register_parser(
      ModuleType::Ts,
      Box::new(JsParser::new(self.unresolved_mark)),
    );
    ctx.context.register_parser(
      ModuleType::Tsx,
      Box::new(JsParser::new(self.unresolved_mark)),
    );
    ctx.context.register_parser(
      ModuleType::Jsx,
      Box::new(JsParser::new(self.unresolved_mark)),
    );

    Ok(())
  }

  fn render_manifest(
    &self,
    _ctx: PluginContext,
    args: rspack_core::RenderManifestArgs,
  ) -> PluginRenderManifestHookOutput {
    let compilation = args.compilation;
    let module_graph = &compilation.module_graph;
    let namespace = &compilation.options.output.unique_name;
    let chunk = compilation
      .chunk_graph
      .chunk_by_id(args.chunk_id)
      .ok_or_else(|| anyhow::format_err!("Not found chunk {:?}", args.chunk_id))?;
    let ordered_modules = chunk.ordered_modules(module_graph);

    let mut module_code_array = ordered_modules
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
              Some(wrap_module_function(source, &module.id))
            } else {
              None
            }
          })
      })
      .collect::<Result<Vec<Option<String>>>>()?;

    // insert chunk wrapper
    module_code_array.insert(
      0,
      Some(get_wrap_chunk_before(
        namespace,
        RSPACK_REGISTER,
        args.chunk_id,
      )),
    );
    module_code_array.push(Some(get_wrap_chunk_after()));

    let code = module_code_array
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

    let output_path = match chunk.kind {
      ChunkKind::Entry { .. } => {
        compilation
          .options
          .output
          .filename
          .render(FilenameRenderOptions {
            filename: Some(args.chunk_id.to_owned()),
            extension: Some(".js".to_owned()),
            id: None,
          })
      }
      ChunkKind::Normal => {
        compilation
          .options
          .output
          .chunk_filename
          .render(FilenameRenderOptions {
            filename: None,
            extension: Some(".js".to_owned()),
            id: Some(format!("static/js/{}", args.chunk_id.to_owned())),
          })
      }
    };

    Ok(vec![RenderManifestEntry::new(
      AssetContent::String(code),
      output_path,
    )])
  }
}

#[derive(Debug)]
struct JsParser {
  unresolved_mark: Mark,
}

impl JsParser {
  fn new(unresolved_mark: Mark) -> Self {
    Self { unresolved_mark }
  }
}

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
        Some(ModuleAst::JavaScript(_ast)) => Ok::<swc_ecma_ast::Program, anyhow::Error>(_ast),
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
          runtime: Some(swc_react::Runtime::Classic),
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
      source_type_list: JS_MODULE_SOURCE_TYPE_LIST,
      unresolved_mark: self.unresolved_mark,
    }))
  }
}
