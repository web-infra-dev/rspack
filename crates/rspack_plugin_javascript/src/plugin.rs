use std::vec;

use crate::module::JS_MODULE_SOURCE_TYPE_LIST;
use crate::utils::{get_wrap_chunk_after, get_wrap_chunk_before, parse_file, wrap_module_function};
use crate::visitors::{ClearMark, DefineScanner, DefineTransform};
use crate::{module::JsModule, utils::get_swc_compiler};
// use anyhow::{Context, Result};
use crate::{RSPACK_REGISTER, RSPACK_REQUIRE};
use rayon::prelude::*;
use rspack_core::{
  get_xxh3_64_hash, AssetContent, BoxModule, ChunkKind, FilenameRenderOptions, ModuleRenderResult,
  ModuleType, ParseModuleArgs, Parser, Plugin, PluginContext, PluginRenderManifestHookOutput,
  RenderManifestEntry, SourceType, TargetPlatform,
};

use rspack_error::{Error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use swc_common::comments::SingleThreadedComments;
use swc_common::Mark;
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
    let mut ordered_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
      &args.chunk_ukey,
      SourceType::JavaScript,
      module_graph,
    );

    ordered_modules.sort_by_key(|m| &m.uri);

    let has_inline_runtime = !TargetPlatform::is_web(&compilation.options.target.platform)
      && matches!(chunk.kind, ChunkKind::Entry { .. });

    let mut module_code_array = ordered_modules
      .par_iter()
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

    let code = module_code_array
      .into_par_iter()
      .flatten()
      .chain([{
        if chunk.kind.is_entry() && !has_inline_runtime {
          // TODO: how do we handle multiple entry modules?
          let entry_module_uri = args
            .compilation
            .chunk_graph
            .get_chunk_entry_modules(&args.chunk_ukey)
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("entry module not found"));
          let entry_module_id = &args
            .compilation
            .module_graph
            .module_by_uri(entry_module_uri)
            .unwrap_or_else(|| panic!("entry module not found"))
            .id;

          compilation
            .runtime
            .generate_rspack_execute(namespace, RSPACK_REQUIRE, entry_module_id)
        } else {
          String::new()
        }
      }])
      .fold(String::new, |mut output, cur| {
        output += &cur;
        output
      })
      .collect::<String>();

    let contenthash = Some(get_xxh3_64_hash(&code).to_string());

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
            contenthash,
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
            contenthash,
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
  ) -> Result<TWithDiagnosticArray<BoxModule>> {
    if !module_type.is_js_like() {
      return Err(Error::InternalError(format!(
        "`module_type` {:?} not supported for `JsParser`",
        module_type
      )));
    }

    let ast_with_diagnostics = parse_file(
      args
        .source
        .try_into_string()
        .map_err(|_| Error::InternalError("Unable to serialize content as string".into()))?,
      args.uri,
      &module_type,
    )?;

    let (ast, diagnostics) = ast_with_diagnostics.split_into_parts();

    let processed_ast = get_swc_compiler().run(|| {
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
      ast: processed_ast,
      uri: args.uri.to_string(),
      module_type,
      source_type_list: JS_MODULE_SOURCE_TYPE_LIST,
      unresolved_mark: self.unresolved_mark,
    });
    Ok(module.with_diagnostic(diagnostics))
  }
}
