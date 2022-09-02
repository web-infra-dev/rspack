use std::vec;

use crate::module::JS_MODULE_SOURCE_TYPE_LIST;
use crate::utils::{get_wrap_chunk_after, get_wrap_chunk_before, parse_file, wrap_module_function};
use crate::visitors::{ClearMark, DefineScanner, DefineTransform};
use crate::{module::JsModule, utils::get_swc_compiler};
// use anyhow::{Context, Result};
use crate::{RSPACK_REGISTER, RSPACK_REQUIRE};
use rayon::prelude::*;
use rspack_core::{
  AssetContent, BoxModule, ChunkKind, ErrorSpan, FilenameRenderOptions, ModuleRenderResult,
  ModuleType, ParseModuleArgs, Parser, Plugin, PluginContext, PluginRenderManifestHookOutput,
  RenderManifestEntry, SourceType, Target, TargetOptions,
};

use rspack_error::{DiagnosticKind, Error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use swc_common::comments::SingleThreadedComments;
use swc_common::{Mark, Spanned};
use swc_ecma_transforms::react::{react, Options as ReactOptions};
use swc_ecma_transforms::{react as swc_react, resolver};
use swc_ecma_visit::{as_folder, FoldWith, VisitWith};

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
  fn apply(&mut self, ctx: PluginContext<&mut rspack_core::ApplyContext>) -> Result<()> {
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
    let chunk = args.chunk();
    let ordered_modules = chunk.ordered_modules(module_graph);

    let has_inline_runtime = matches!(
      &compilation.options.target,
      Target::Target(TargetOptions::WebWorker),
    ) && matches!(chunk.kind, ChunkKind::Entry { .. });

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

    if !has_inline_runtime {
      // insert chunk wrapper
      module_code_array.insert(
        0,
        Some(get_wrap_chunk_before(
          namespace,
          RSPACK_REGISTER,
          &args.chunk().id.to_owned(),
        )),
      );
      module_code_array.push(Some(get_wrap_chunk_after()));
    }

    let entry_module_id = ordered_modules
      .last()
      .ok_or_else(|| anyhow::format_err!("TODO:"))?
      .id
      .as_str();

    let execute_code =
      compilation
        .runtime
        .generate_rspack_execute(namespace, RSPACK_REQUIRE, entry_module_id);

    let mut code = module_code_array
      .into_par_iter()
      .flatten()
      .chain([{
        if chunk.kind.is_entry() && !has_inline_runtime {
          execute_code.clone()
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
            filename: Some(args.chunk().id.to_owned()),
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
            id: Some(format!("static/js/{}", args.chunk().id.to_owned())),
          })
      }
    };

    if has_inline_runtime {
      code = compilation
        .runtime
        .generate_with_inline_modules(&code, &execute_code);
    }

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
  ) -> Result<TWithDiagnosticArray<BoxModule>> {
    if !module_type.is_js_like() {
      return Err(Error::InternalError(format!(
        "`module_type` {:?} not supported for `JsParser`",
        module_type
      )));
    }

    // let ast = {
    //   match args.ast {
    //     Some(ModuleAst::JavaScript(_ast)) => Ok::<_, anyhow::Error>(_ast),
    //     None => {
    //       if let Some(content) = args.source {
    //         Ok(parse_file(
    //           content
    //             .try_into_string()
    //             .context("Unable to serialize content as string which is required by plugin css")?,
    //           args.uri,
    //           &module_type,
    //         ))
    //       } else {
    //         anyhow::bail!(
    //           "ast and source is both empty for {}, or content type does not match {:?}",
    //           args.uri,
    //           args.source
    //         )
    //       }
    //     }
    //     _ => anyhow::bail!("not supported ast {:?} for js parser", args.ast),
    //   }
    // }?;

    let ast = parse_file(
      args
        .source
        .try_into_string()
        .map_err(|_| Error::InternalError("Unable to serialize content as string".into()))?,
      args.uri,
      &module_type,
    )
    .map_err(|err| {
      // Convert `swc_ecma_parser::error::Error` to `rspack_error::Error`
      Error::BatchErrors(
        err
          .into_iter()
          .map(|e| ecma_parse_error_to_diagnostic(e, args.uri, &module_type))
          .collect(),
      )
    })?;

    let ast = get_swc_compiler().run(|| {
      let defintions = &args.options.define;
      let mut define_scanner = DefineScanner::new(defintions);
      // TODO: find more suitable position.
      ast.visit_with(&mut define_scanner);
      let mut define_transform = DefineTransform::new(defintions, define_scanner);

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

      // TODO: the order
      let ast = ast.fold_with(&mut define_transform);
      let ast = ast.fold_with(&mut resolver(Mark::new(), top_level_mark, false));
      let ast = ast.fold_with(&mut react_folder);
      ast.fold_with(&mut as_folder(ClearMark))
    });
    let module: BoxModule = Box::new(JsModule {
      ast,
      uri: args.uri.to_string(),
      module_type,
      source_type_list: JS_MODULE_SOURCE_TYPE_LIST,
      unresolved_mark: self.unresolved_mark,
    });
    Ok(module.with_empty_diagnostic())
  }
}

pub fn ecma_parse_error_to_diagnostic(
  error: swc_ecma_parser::error::Error,
  path: &str,
  module_type: &ModuleType,
) -> Error {
  let (file_type, diagnostic_kind) = match module_type {
    ModuleType::Js => ("JavaScript", DiagnosticKind::JavaScript),
    ModuleType::Jsx => ("JSX", DiagnosticKind::Jsx),
    ModuleType::Tsx => ("TSX", DiagnosticKind::Tsx),
    ModuleType::Ts => ("Typescript", DiagnosticKind::Typescript),
    _ => unreachable!(),
  };
  let message = error.kind().msg().to_string();
  let span: ErrorSpan = error.span().into();
  let traceable_error = rspack_error::TraceableError::from_path(
    path.to_string(),
    span.start as usize,
    span.end as usize,
    format!("{} parsing error", file_type),
    message,
  )
  .with_kind(diagnostic_kind);
  rspack_error::Error::TraceableError(traceable_error)
  //Use this `Error` convertion could avoid eagerly clone source file.
}
