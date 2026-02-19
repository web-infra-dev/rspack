/**
 * Some code is modified based on
 * https://github.com/swc-project/swc/blob/5dacaa174baaf6bf40594d79d14884c8c2fc0de2/crates/swc/src/lib.rs
 * Apache-2.0 licensed
 * Author Donny/강동윤
 * Copyright (c)
 */
use std::{fs::File, path::PathBuf, sync::Arc};

use anyhow::{Context, bail};
use base64::prelude::*;
use indoc::formatdoc;
use rspack_error::Result;
use rspack_util::{source_map::SourceMapKind, swc::minify_file_comments};
use swc_config::{is_module::IsModule, merge::Merge};
pub use swc_core::base::config::Options as SwcOptions;
use swc_core::{
  base::{
    BoolOr,
    config::{
      BuiltInput, Config, InputSourceMap, JsMinifyCommentOption, OutputCharset, SourceMapsConfig,
    },
    sourcemap,
  },
  common::{
    FileName, GLOBALS, Mark, SourceFile, SourceMap,
    comments::{Comments, SingleThreadedComments},
    errors::Handler,
  },
  ecma::{
    ast::{EsVersion, Pass, Program},
    parser::{
      Syntax, TsSyntax, parse_file_as_commonjs, parse_file_as_module, parse_file_as_program,
      parse_file_as_script,
    },
    transforms::base::helpers::{self, Helpers},
  },
};
use swc_error_reporters::handler::try_with_handler;
use url::Url;

use super::{
  JavaScriptCompiler, TransformOutput,
  stringify::{PrintOptions, SourceMapConfig},
};

impl JavaScriptCompiler {
  /// Transforms the given JavaScript source code according to the provided options and source map kind.
  #[allow(clippy::too_many_arguments)]
  pub fn transform<'a, S, P>(
    &self,
    source: S,
    filename: Option<Arc<FileName>>,
    comments: std::rc::Rc<SingleThreadedComments>,
    options: SwcOptions,
    module_source_map_kind: Option<SourceMapKind>,
    inspect_parsed_ast: impl FnOnce(&Program, Mark),
    before_pass: impl FnOnce(&Program) -> P + 'a,
  ) -> Result<TransformOutput>
  where
    P: Pass + 'a,
    S: Into<String>,
  {
    let fm = self.cm.new_source_file(
      filename.unwrap_or_else(|| Arc::new(FileName::Anon)),
      source.into(),
    );
    let javascript_transformer =
      JavaScriptTransformer::new(self.cm.clone(), fm, comments, self, options)?;

    javascript_transformer.transform(inspect_parsed_ast, before_pass, module_source_map_kind)
  }
}

struct JavaScriptTransformer<'a> {
  cm: Arc<SourceMap>,
  fm: Arc<SourceFile>,
  comments: std::rc::Rc<SingleThreadedComments>,
  options: SwcOptions,
  javascript_compiler: &'a JavaScriptCompiler,
  helpers: Helpers,
  config: Config,
}

const SWC_MIETTE_DIAGNOSTIC_CODE: &str = "Builtin swc-loader error";

impl<'a> JavaScriptTransformer<'a> {
  pub fn new(
    cm: Arc<SourceMap>,
    fm: Arc<SourceFile>,
    comments: std::rc::Rc<SingleThreadedComments>,
    compiler: &'a JavaScriptCompiler,
    mut options: SwcOptions,
  ) -> Result<Self> {
    GLOBALS.set(&compiler.globals, || {
      let top_level_mark = Mark::new();
      let unresolved_mark = Mark::new();
      options.top_level_mark = Some(top_level_mark);
      options.unresolved_mark = Some(unresolved_mark);
    });

    let config = get_swc_config_from_file(&fm.name);
    let helpers = GLOBALS.set(&compiler.globals, || {
      let mut external_helpers = options.config.jsc.external_helpers;
      external_helpers.merge(config.jsc.external_helpers);
      Helpers::new(external_helpers.into())
    });

    Ok(Self {
      cm,
      fm,
      javascript_compiler: compiler,
      options,
      helpers,
      config,
      comments,
    })
  }

  fn transform<P>(
    self,
    inspect_parsed_ast: impl FnOnce(&Program, Mark),
    before_pass: impl FnOnce(&Program) -> P + 'a,
    module_source_map_kind: Option<SourceMapKind>,
  ) -> Result<TransformOutput>
  where
    P: Pass + 'a,
  {
    let mut built_input = self.parse_built_input(before_pass)?;

    let target = built_input.target;
    let source_map_kind: SourceMapKind = match self.options.config.source_maps {
      Some(SourceMapsConfig::Bool(false)) => SourceMapKind::empty(),
      _ => module_source_map_kind.unwrap_or_default(),
    };
    let minify = built_input.minify;
    let source_map_config = SourceMapConfig {
      enable: source_map_kind.source_map(),
      inline_sources_content: source_map_kind.source_map(),
      emit_columns: !source_map_kind.cheap(),
      names: Default::default(),
    };

    let input_source_map = self.input_source_map(&built_input.input_source_map)?;

    let diagnostics = self.transform_with_built_input(&mut built_input, inspect_parsed_ast)?;
    let ascii_only = built_input
      .output
      .charset
      .as_ref()
      .is_some_and(|v| matches!(v, OutputCharset::Ascii));

    let print_options = PrintOptions {
      source_len: self.fm.byte_length(),
      source_map: self.cm.clone(),
      target,
      source_map_config,
      input_source_map: input_source_map.as_ref(),
      minify,
      comments: Some(&self.comments as &dyn Comments),
      preamble: &built_input.output.preamble,
      ascii_only,
      inline_script: built_input.codegen_inline_script,
    };

    self
      .javascript_compiler
      .print(&built_input.program, print_options)
      .map(|o| o.with_diagnostics(diagnostics))
  }

  fn parse_js(
    &self,
    fm: Arc<SourceFile>,
    handler: &Handler,
    target: EsVersion,
    syntax: Syntax,
    is_module: IsModule,
    comments: Option<&dyn Comments>,
  ) -> Result<Program, anyhow::Error> {
    let mut error = false;
    let mut errors = vec![];

    let program_result = match is_module {
      IsModule::Bool(true) => {
        parse_file_as_module(&fm, syntax, target, comments, &mut errors).map(Program::Module)
      }
      IsModule::Bool(false) => {
        parse_file_as_script(&fm, syntax, target, comments, &mut errors).map(Program::Script)
      }
      IsModule::Unknown => parse_file_as_program(&fm, syntax, target, comments, &mut errors),
      IsModule::CommonJS => {
        parse_file_as_commonjs(&fm, syntax, target, comments, &mut errors).map(Program::Script)
      }
    };

    for e in errors {
      e.into_diagnostic(handler).emit();
      error = true;
    }

    let res = program_result.map_err(|e| {
      e.into_diagnostic(handler).emit();
      anyhow::Error::msg("Syntax Error")
    });

    if error {
      return Err(anyhow::anyhow!("Syntax Error"));
    }
    res
  }

  fn parse_built_input<P>(
    &'a self,
    before_pass: impl FnOnce(&Program) -> P + 'a,
  ) -> Result<BuiltInput<impl Pass + 'a>>
  where
    P: Pass + 'a,
  {
    self.run(|| {
      try_with_handler(self.cm.clone(), Default::default(), |handler| {
        self.options.build_as_input(
          &self.cm.clone(),
          &self.fm.name,
          move |syntax, target, is_module| {
            self.parse_js(
              self.fm.clone(),
              handler,
              target,
              syntax,
              is_module,
              Some(&self.comments).map(|c| c as &dyn Comments),
            )
          },
          self.options.output_path.as_deref(),
          self.options.source_root.clone(),
          self.options.source_file_name.clone(),
          self.config.source_map_ignore_list.clone(),
          handler,
          Some(self.config.clone()),
          Some(&self.comments),
          before_pass,
        )
      })
      .map_err(|e| e.to_pretty_error().into())
    })
  }

  fn run<R>(&self, op: impl FnOnce() -> R) -> R {
    GLOBALS.set(&self.javascript_compiler.globals, op)
  }

  fn transform_with_built_input(
    &self,
    built_input: &mut BuiltInput<impl Pass>,
    inspect_parsed_ast: impl FnOnce(&Program, Mark),
  ) -> Result<Vec<String>> {
    let mut diagnostics = vec![];
    let result = self.run(|| {
      helpers::HELPERS.set(&self.helpers, || {
        inspect_parsed_ast(&built_input.program, built_input.unresolved_mark);

        let result = try_with_handler(self.cm.clone(), Default::default(), |handler| {
          // Apply external plugin passes to the Program AST.
          // External plugins may emit warnings or inject helpers,
          // so we need a handler to properly process them.
          built_input.pass.process(&mut built_input.program);
          diagnostics.extend(handler.take_diagnostics());

          Ok(())
        });

        result.map_err(|err| {
          let swc_diagnostics = err.diagnostics();

          if swc_diagnostics.iter().any(|d| match &d.code {
            Some(code) => {
              // reference to:
              //    https://github.com/swc-project/swc/blob/v1.11.21/crates/swc/src/plugin.rs#L187
              //    https://github.com/swc-project/swc/blob/v1.11.21/crates/swc/src/plugin.rs#L200
              match code {
                swc_core::common::errors::DiagnosticId::Error(e) => e.contains("plugin"),
                swc_core::common::errors::DiagnosticId::Lint(_) => false,
              }
            }
            None => false,
          }) {
            // swc errors includes plugin error;
            let error_msg = err.to_pretty_string();
            let swc_core_version = rspack_workspace::rspack_swc_core_version!();
            // FIXME: with_help has bugs, use with_help when diagnostic print is fixed
            let help_msg = formatdoc!{"
              The version of the SWC Wasm plugin you're using might not be compatible with `builtin:swc-loader`.
              The `swc_core` version of the current `rspack_core` is {swc_core_version}. 
              Please check the `swc_core` version of SWC Wasm plugin to make sure these versions are within the compatible range.
              See this guide as a reference for selecting SWC Wasm plugin versions: https://rspack.rs/errors/swc-plugin-version"};
            let mut error = rspack_error::error!(format!("{error_msg}{help_msg}"));
            error.code = Some(SWC_MIETTE_DIAGNOSTIC_CODE.into());
            error
          } else {
            let error_msg = err.to_pretty_string();
            let mut error = rspack_error::error!(error_msg);
            error.code = Some(SWC_MIETTE_DIAGNOSTIC_CODE.into());
            error
          }
        })
      })
    });

    if let Some(comments) = &built_input.comments {
      let preserve_annotations = match &built_input.preserve_comments {
        BoolOr::Bool(true) | BoolOr::Data(JsMinifyCommentOption::PreserveAllComments) => true,
        BoolOr::Data(JsMinifyCommentOption::PreserveSomeComments) => false,
        BoolOr::Bool(false) => false,
        BoolOr::Data(JsMinifyCommentOption::PreserveRegexComments { .. }) => false,
      };

      minify_file_comments(
        comments,
        &built_input.preserve_comments,
        preserve_annotations,
      );
    }

    result.map(|_| diagnostics)
  }

  pub fn input_source_map(
    &self,
    input_src_map: &InputSourceMap,
  ) -> Result<Option<sourcemap::SourceMap>, anyhow::Error> {
    let fm = &self.fm;
    let name = &self.fm.name;
    let read_inline_sourcemap =
      |data_url: Option<&str>| -> Result<Option<sourcemap::SourceMap>, anyhow::Error> {
        match data_url {
          Some(data_url) => {
            let url = Url::parse(data_url)
              .with_context(|| format!("failed to parse inline source map url\n{data_url}"))?;

            let idx = match url.path().find("base64,") {
              Some(v) => v,
              None => {
                bail!("failed to parse inline source map: not base64: {url:?}")
              }
            };

            let content = url.path()[idx + "base64,".len()..].trim();

            let res = BASE64_STANDARD
              .decode(content.as_bytes())
              .context("failed to decode base64-encoded source map")?;

            Ok(Some(sourcemap::SourceMap::from_slice(&res).context(
              "failed to read input source map from inlined base64 encoded \
                                string",
            )?))
          }
          None => {
            bail!("failed to parse inline source map: `sourceMappingURL` not found")
          }
        }
      };

    let read_file_sourcemap =
      |data_url: Option<&str>| -> Result<Option<sourcemap::SourceMap>, anyhow::Error> {
        match name.as_ref() {
          FileName::Real(filename) => {
            let dir = match filename.parent() {
              Some(v) => v,
              None => {
                bail!("unexpected: root directory is given as a input file")
              }
            };

            let map_path = match data_url {
              Some(data_url) => {
                let mut map_path = dir.join(data_url);
                if !map_path.exists() {
                  // Old behavior. This check would prevent
                  // regressions.
                  // Perhaps it shouldn't be supported. Sometimes
                  // developers don't want to expose their source
                  // code.
                  // Map files are for internal troubleshooting
                  // convenience.
                  map_path = PathBuf::from(format!("{}.map", filename.display()));
                  if !map_path.exists() {
                    bail!(
                      "failed to find input source map file {:?} in \
                                                  {:?} file",
                      map_path.display(),
                      filename.display()
                    )
                  }
                }

                Some(map_path)
              }
              None => {
                // Old behavior.
                let map_path = PathBuf::from(format!("{}.map", filename.display()));
                if map_path.exists() {
                  Some(map_path)
                } else {
                  None
                }
              }
            };

            match map_path {
              Some(map_path) => {
                let path = map_path.display().to_string();
                let file = File::open(&path);

                // Old behavior.
                let file = file?;

                Ok(Some(sourcemap::SourceMap::from_reader(file).with_context(
                  || {
                    format!(
                      "failed to read input source map
                                  from file at {path}"
                    )
                  },
                )?))
              }
              None => Ok(None),
            }
          }
          _ => Ok(None),
        }
      };

    let read_sourcemap = || -> Option<sourcemap::SourceMap> {
      let s = "sourceMappingURL=";
      let idx = fm.src.rfind(s);

      let data_url = idx.map(|idx| {
        let data_idx = idx + s.len();
        if let Some(end) = fm.src[data_idx..].find('\n').map(|i| i + data_idx + 1) {
          &fm.src[data_idx..end]
        } else {
          &fm.src[data_idx..]
        }
      });

      match read_inline_sourcemap(data_url) {
        Ok(r) => r,
        Err(_err) => {
          // Load original source map if possible
          read_file_sourcemap(data_url).unwrap_or(None)
        }
      }
    };

    // Load original source map
    match input_src_map {
      InputSourceMap::Bool(false) => Ok(None),
      InputSourceMap::Bool(true) => Ok(read_sourcemap()),
      InputSourceMap::Str(s) => {
        if s == "inline" {
          Ok(read_sourcemap())
        } else {
          // Load source map passed by user
          Ok(Some(
            sourcemap::SourceMap::from_slice(s.as_bytes())
              .context("failed to read input source map from user-provided sourcemap")?,
          ))
        }
      }
    }
  }
}

fn get_swc_config_from_file(filename: &FileName) -> Config {
  let filename_path = match filename {
    FileName::Real(p) => Some(p.as_path()),
    _ => return Config::default(),
  };

  let filename_ext = match filename_path {
    Some(p) => p.extension().and_then(|ext| ext.to_str()),
    None => return Config::default(),
  };

  let mut config = Config::default();
  match filename_ext {
    Some("tsx") => {
      config.jsc.syntax = Some(Syntax::Typescript(TsSyntax {
        tsx: true,
        ..Default::default()
      }))
    }
    Some("cts" | "mts") => {
      config.jsc.syntax = Some(Syntax::Typescript(TsSyntax {
        tsx: false,
        disallow_ambiguous_jsx_like: true,
        ..Default::default()
      }))
    }
    Some("ts") => {
      config.jsc.syntax = Some(Syntax::Typescript(TsSyntax {
        tsx: false,
        ..Default::default()
      }))
    }
    _ => {}
  }

  config
}
