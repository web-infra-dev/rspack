mod minify;

use std::{collections::HashMap, hash::Hash, sync::Mutex};

use async_trait::async_trait;
use minify::{match_object, minify};
use rayon::prelude::*;
use rspack_core::{
  rspack_sources::{
    MapOptions, RawSource, SourceExt, SourceMap, SourceMapSource, SourceMapSourceOptions,
  },
  AssetInfo, CompilationAsset, JsChunkHashArgs, Plugin, PluginContext, PluginJsChunkHashHookOutput,
  PluginProcessAssetsOutput, ProcessAssetsArgs,
};
use rspack_error::{internal_error, Result};
use rspack_regex::RspackRegex;
use rspack_util::try_any_sync;
use swc_config::config_types::BoolOrDataConfig;
use swc_ecma_minifier::option::{
  terser::{TerserCompressorOptions, TerserEcmaVersion},
  MangleOptions,
};

#[derive(Debug, Clone, Default, Hash)]
pub struct SwcJsMinimizerRspackPluginOptions {
  pub passes: usize,
  pub drop_console: bool,
  pub keep_class_names: bool,
  pub keep_fn_names: bool,
  pub pure_funcs: Vec<String>,
  pub extract_comments: Option<String>,
  pub ascii_only: bool,
  pub comments: String,
  pub test: Option<SwcJsMinimizerRules>,
  pub include: Option<SwcJsMinimizerRules>,
  pub exclude: Option<SwcJsMinimizerRules>,
}

#[derive(Debug, Clone, Hash)]
pub enum SwcJsMinimizerRule {
  String(String),
  Regexp(RspackRegex),
}

impl SwcJsMinimizerRule {
  pub fn try_match(&self, data: &str) -> rspack_error::Result<bool> {
    match self {
      Self::String(s) => Ok(data.starts_with(s)),
      Self::Regexp(r) => Ok(r.test(data)),
    }
  }
}

#[derive(Debug, Clone, Hash)]
pub enum SwcJsMinimizerRules {
  String(String),
  Regexp(rspack_regex::RspackRegex),
  Array(Vec<SwcJsMinimizerRule>),
}

impl SwcJsMinimizerRules {
  pub fn try_match(&self, data: &str) -> rspack_error::Result<bool> {
    match self {
      Self::String(s) => Ok(data.starts_with(s)),
      Self::Regexp(r) => Ok(r.test(data)),
      Self::Array(l) => try_any_sync(l, |i| i.try_match(data)),
    }
  }
}

#[derive(Debug)]
pub struct SwcJsMinimizerRspackPlugin {
  options: SwcJsMinimizerRspackPluginOptions,
}

impl SwcJsMinimizerRspackPlugin {
  pub fn new(options: SwcJsMinimizerRspackPluginOptions) -> Self {
    Self { options }
  }
}

#[async_trait]
impl Plugin for SwcJsMinimizerRspackPlugin {
  fn name(&self) -> &'static str {
    "rspack.SwcJsMinimizerRspackPlugin"
  }

  async fn process_assets_stage_optimize_size(
    &self,
    _ctx: PluginContext,
    args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    let compilation = args.compilation;
    let minify_options = &self.options;
    let is_module = compilation
      .options
      .output
      .library
      .as_ref()
      .is_some_and(|library| library.library_type == "module");

    // collect all extracted comments info
    let all_extracted_comments = Mutex::new(HashMap::new());
    let extract_comments_option = &minify_options.extract_comments.clone();
    let emit_source_map_columns = !compilation.options.devtool.cheap();
    let compress = TerserCompressorOptions {
      passes: minify_options.passes,
      drop_console: minify_options.drop_console,
      pure_funcs: minify_options.pure_funcs.clone(),
      ..Default::default()
    };

    let mangle = MangleOptions {
      keep_class_names: minify_options.keep_class_names,
      keep_fn_names: minify_options.keep_fn_names,
      ..Default::default()
    };

    let comments = match minify_options.comments.as_str() {
      "false" => JsMinifyCommentOption::False,
      "all" => JsMinifyCommentOption::PreserveAllComments,
      "some" => JsMinifyCommentOption::PreserveSomeComments,
      _ => JsMinifyCommentOption::False,
    };

    let format = JsMinifyFormatOptions {
      ascii_only: minify_options.ascii_only,
      comments,
      ..Default::default()
    };

    let result = compilation
    .assets_mut()
    .par_iter_mut()
    .filter(|(filename, original)| {
      if !(filename.ends_with(".js") || filename.ends_with(".cjs") || filename.ends_with(".mjs")) {
        return false
      }

      let is_matched = match_object(minify_options, filename)
        .unwrap_or(false);

      if !is_matched || original.get_info().minimized {
        return false
      }

      true
    })
    .try_for_each(|(filename, original)| -> Result<()>  {
        if let Some(original_source) = original.get_source() {
          let input = original_source.source().to_string();
          let input_source_map = original_source.map(&MapOptions::default());
          let js_minify_options = JsMinifyOptions {
            compress: BoolOrDataConfig::from_obj(compress.clone()),
            mangle: BoolOrDataConfig::from_obj(mangle.clone()),
            format: format.clone(),
            source_map: BoolOrDataConfig::from_bool(input_source_map.is_some()),
            inline_sources_content: true, /* Using true so original_source can be None in SourceMapSource */
            emit_source_map_columns,
            module: is_module,
            ..Default::default()
          };
          let output = match minify(
            &js_minify_options,
            input,
            filename,
            &all_extracted_comments,
            extract_comments_option,
          ) {
            Ok(r) => r,
            Err(e) => {
              return Err(e)
            }
          };
          let source = if let Some(map) = &output.map {
            SourceMapSource::new(SourceMapSourceOptions {
              value: output.code,
              name: filename,
              source_map: SourceMap::from_json(map).map_err(|e| internal_error!(e.to_string()))?,
              original_source: None,
              inner_source_map: input_source_map,
              remove_original_source: true,
            })
            .boxed()
          } else {
            RawSource::from(output.code).boxed()
          };
          original.set_source(Some(source));
          original.get_info_mut().minimized = true;
        }

        Ok(())
    });

    match result {
      Ok(_) => {}
      Err(e) => compilation.push_batch_diagnostic(e.into()),
    };

    // write all extracted comments to assets
    all_extracted_comments
      .lock()
      .expect("all_extracted_comments lock failed")
      .clone()
      .into_iter()
      .for_each(|(_, comments)| {
        compilation.emit_asset(
          comments.comments_file_name,
          CompilationAsset {
            source: Some(comments.source),
            info: AssetInfo {
              minimized: true,
              ..Default::default()
            },
          },
        )
      });

    Ok(())
  }

  fn js_chunk_hash(
    &self,
    _ctx: PluginContext,
    args: &mut JsChunkHashArgs,
  ) -> PluginJsChunkHashHookOutput {
    self.name().hash(&mut args.hasher);
    self.options.hash(&mut args.hasher);
    Ok(())
  }
}

#[derive(Debug, Clone, Default)]
pub enum JsMinifyCommentOption {
  #[default]
  False,
  PreserveSomeComments,
  PreserveAllComments,
}

#[derive(Debug, Clone, Default)]
pub struct JsMinifyOptions {
  pub compress: BoolOrDataConfig<TerserCompressorOptions>,
  pub mangle: BoolOrDataConfig<MangleOptions>,
  pub format: JsMinifyFormatOptions,
  pub ecma: TerserEcmaVersion,
  pub keep_class_names: bool,
  pub keep_fn_names: bool,
  pub module: bool,
  pub safari10: bool,
  pub toplevel: bool,
  pub source_map: BoolOrDataConfig<TerserSourceMapOption>,
  pub output_path: Option<String>,
  pub inline_sources_content: bool,
  pub emit_source_map_columns: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TerserSourceMapOption {
  pub filename: Option<String>,
  pub url: Option<String>,
  pub root: Option<String>,
  pub content: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct JsMinifyFormatOptions {
  pub ascii_only: bool,
  pub beautify: bool,
  pub braces: bool,
  pub comments: JsMinifyCommentOption,
  pub ecma: usize,
  pub indent_level: Option<usize>,
  pub indent_start: bool,
  pub inline_script: bool,
  pub keep_numbers: bool,
  pub keep_quoted_props: bool,
  pub max_line_len: usize,
  pub preamble: String,
  pub quote_keys: bool,
  pub quote_style: usize,
  pub preserve_annotations: bool,
  pub safari10: bool,
  pub semicolons: bool,
  pub shebang: bool,
  pub webkit: bool,
  pub wrap_iife: bool,
  pub wrap_func_args: bool,
}
