/**
 * Some code is modified based on
 * https://github.com/swc-project/swc/blob/5dacaa174baaf6bf40594d79d14884c8c2fc0de2/crates/swc/src/lib.rs
 * Apache-2.0 licensed
 * Author Donny/강동윤
 * Copyright (c)
 */
use std::{cell::RefCell, io, path::PathBuf, rc::Rc, sync::Arc};

use anyhow::{Context, bail};
use indoc::formatdoc;
use rspack_error::Result;
use rspack_sources::SourceMap as RspackSourceMap;
use rspack_util::{base64, source_map::SourceMapKind};
use rustc_hash::FxHashSet as HashSet;
use swc_config::{is_module::IsModule, merge::Merge};
pub use swc_core::base::config::Options as SwcOptions;
use swc_core::{
  base::{
    BoolOr,
    config::{
      BuiltInput, Config, InputSourceMap, JsMinifyCommentOption, OutputCharset, SourceMapsConfig,
    },
  },
  common::{
    FileName, GLOBALS, Mark, SourceFile, SourceMap,
    comments::{Comments, SingleThreadedComments},
    errors::{DiagnosticId, Handler},
  },
  ecma::{
    ast::{
      EsVersion, ExportAll, ImportDecl, NamedExport, Pass, Program, TsExternalModuleRef,
      TsImportType,
    },
    codegen::{
      self, Emitter, Node,
      text_writer::{self, WriteJs},
    },
    parser::{
      Syntax, TsSyntax, parse_file_as_commonjs, parse_file_as_module, parse_file_as_program,
      parse_file_as_script,
    },
    transforms::base::{
      helpers::{self, Helpers},
      resolver,
    },
    visit::{Visit, VisitMutWith, VisitWith},
  },
};
use swc_error_reporters::handler::try_with_handler;
use swc_typescript::fast_dts::FastDts;
use url::Url;

use super::{
  IsolatedDtsTransformOutput, JavaScriptCompiler, TransformOutput,
  minify::minify_file_comments,
  stringify::{PrintOptions, SourceMapConfig},
};
use crate::error::swc_diagnostics_to_rspack_error;

const SWC_SYNTAX_ERROR_CODE: &str = "Syntax Error";

pub type InputSourceMapResult = anyhow::Result<Option<RspackSourceMap<'static>>>;

/// The next actual I/O operation required by the shared map-selection policy.
pub enum InputSourceMapRequest {
  Metadata(PathBuf),
  Read(PathBuf),
}

/// Resolves a map once, independently of whether its filesystem is synchronous or async.
/// No SWC transformer, comments or thread-local compiler state is held while awaiting I/O.
pub struct InputSourceMapResolver {
  request: Option<InputSourceMapRequest>,
  fallback: Option<PathBuf>,
  result: InputSourceMapResult,
  automatic_lookup: Option<bool>,
}

impl InputSourceMapResolver {
  pub fn new(source: &str, name: &FileName, input_src_map: &InputSourceMap) -> Self {
    let mut resolver = Self {
      request: None,
      fallback: None,
      result: Ok(None),
      automatic_lookup: None,
    };
    match input_src_map {
      InputSourceMap::Bool(false) => return resolver,
      InputSourceMap::Str(s) if s != "inline" => {
        resolver.result = RspackSourceMap::from_json(s.clone())
          .context("failed to read input source map from user-provided sourcemap")
          .map(Some);
        return resolver;
      }
      _ => {}
    }

    let marker = "sourceMappingURL=";
    let idx = source.rfind(marker);
    resolver.automatic_lookup = Some(idx.is_some());
    let data_url = idx.map(|idx| {
      let start = idx + marker.len();
      match source[start..].find('\n') {
        Some(end) => &source[start..start + end + 1],
        None => &source[start..],
      }
    });
    if let Ok(map) = read_inline_sourcemap(data_url) {
      resolver.result = Ok(Some(map));
      return resolver;
    }

    if let FileName::Real(filename) = name
      && let Some(dir) = filename.parent()
    {
      let adjacent = PathBuf::from(format!("{}.map", filename.display()));
      let path = if let Some(data_url) = data_url {
        resolver.fallback = Some(adjacent);
        dir.join(data_url)
      } else {
        adjacent
      };
      resolver.request = Some(InputSourceMapRequest::Metadata(path));
    }
    resolver
  }

  pub fn automatic_lookup(&self) -> Option<bool> {
    self.automatic_lookup
  }

  pub fn next_request(&mut self) -> Option<InputSourceMapRequest> {
    self.request.take()
  }

  pub fn metadata_result(&mut self, path: PathBuf, found: bool) {
    self.request = if found {
      Some(InputSourceMapRequest::Read(path))
    } else {
      self.fallback.take().map(InputSourceMapRequest::Metadata)
    };
  }

  pub fn read_result(&mut self, content: io::Result<Vec<u8>>) {
    // Automatic lookup historically ignores file read/parse errors and does not try another
    // candidate after a successful metadata probe. Explicit JSON errors still propagate.
    self.result = Ok(
      content
        .ok()
        .and_then(|bytes| RspackSourceMap::from_bytes(bytes).ok()),
    );
  }

  pub fn finish(self) -> InputSourceMapResult {
    self.result
  }
}

fn read_inline_sourcemap(data_url: Option<&str>) -> anyhow::Result<RspackSourceMap<'static>> {
  let Some(data_url) = data_url else {
    bail!("failed to parse inline source map: `sourceMappingURL` not found")
  };
  let url = Url::parse(data_url)
    .with_context(|| format!("failed to parse inline source map url\n{data_url}"))?;
  let Some(idx) = url.path().find("base64,") else {
    bail!("failed to parse inline source map: not base64: {url:?}")
  };
  let content = url.path()[idx + "base64,".len()..].trim();
  let bytes = base64::decode_to_vec(content.as_bytes())
    .context("failed to decode base64-encoded source map")?;
  RspackSourceMap::from_bytes(bytes)
    .context("failed to read input source map from inlined base64 encoded string")
}

fn native_input_source_map(
  source: &str,
  name: &FileName,
  option: &InputSourceMap,
) -> InputSourceMapResult {
  let mut resolver = InputSourceMapResolver::new(source, name, option);
  while let Some(request) = resolver.next_request() {
    match request {
      InputSourceMapRequest::Metadata(path) => {
        let found = std::fs::metadata(&path).is_ok();
        resolver.metadata_result(path, found);
      }
      InputSourceMapRequest::Read(path) => resolver.read_result(std::fs::read(path)),
    }
  }
  resolver.finish()
}

fn emit_syntax_error(error: swc_core::ecma::parser::error::Error, handler: &Handler) {
  error
    .into_diagnostic(handler)
    .code(DiagnosticId::Error(SWC_SYNTAX_ERROR_CODE.into()))
    .emit();
}

impl JavaScriptCompiler {
  /// Transforms the given JavaScript source code according to the provided options and source map kind.
  /// A supplied input-map result completes acquisition, including an absent map or an error.
  /// Only `None` requests native filesystem acquisition after parsing.
  #[allow(clippy::too_many_arguments)]
  pub fn transform<'a, S, P>(
    &self,
    source: S,
    filename: Option<Arc<FileName>>,
    comments: std::rc::Rc<SingleThreadedComments>,
    options: SwcOptions,
    module_source_map_kind: Option<SourceMapKind>,
    input_source_map: Option<InputSourceMapResult>,
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

    javascript_transformer.transform(
      inspect_parsed_ast,
      before_pass,
      module_source_map_kind,
      input_source_map,
    )
  }

  pub fn emit_isolated_dts(
    &self,
    program: &Program,
    filename: Arc<FileName>,
    unresolved_mark: Mark,
    target: EsVersion,
    comments: &SingleThreadedComments,
  ) -> Result<IsolatedDtsTransformOutput> {
    self.run(|| {
      let (leading, trailing) = comments.borrow_all();
      let comments = SingleThreadedComments::from_leading_and_trailing(
        Rc::new(RefCell::new(leading.clone())),
        Rc::new(RefCell::new(trailing.clone())),
      );

      // FastDts mutates the cloned program into declaration form. Re-print it with
      // declaration-safe codegen settings instead of reusing the JS output pipeline.
      let mut program = program.clone();
      let mut checker = FastDts::new(filename, unresolved_mark, Default::default());
      let diagnostics = match checker.transform(&mut program) {
        issues if issues.is_empty() => Vec::new(),
        issues => {
          let result = try_with_handler(self.cm.clone(), Default::default(), |handler| {
            for issue in issues {
              handler
                .struct_span_err(issue.range.span, &issue.message)
                .emit();
            }

            Ok(())
          });

          match result {
            Ok(()) => Vec::new(),
            Err(error) => vec![error.to_pretty_string()],
          }
        }
      };

      // Collect declaration references from the FastDts output AST before codegen.
      // Rslib uses these references to complete type-only declaration outputs without
      // parsing the generated .d.ts string again.
      let mut collector = DtsReferenceCollector::default();
      program.visit_with(&mut collector);
      let references = collector.references;

      let code = {
        let mut buf = Vec::new();
        {
          let wr = Box::new(text_writer::JsWriter::new(
            self.cm.clone(),
            "\n",
            &mut buf,
            None,
          )) as Box<dyn WriteJs>;
          let mut emitter = Emitter {
            cfg: codegen::Config::default().with_target(target),
            comments: Some(&comments as &dyn Comments),
            cm: self.cm.clone(),
            wr,
          };
          program.emit_with(&mut emitter)?;
        }
        // SAFETY: SWC will emit valid utf8 for sure
        unsafe { String::from_utf8_unchecked(buf) }
      };

      Ok(IsolatedDtsTransformOutput {
        code,
        references,
        diagnostics,
      })
    })
  }

  /// Generate isolated declaration output from a source string for callers that
  /// are outside the normal loader transform path.
  pub fn emit_isolated_dts_from_source(
    &self,
    source: String,
    filename: Arc<FileName>,
    syntax: Syntax,
    target: EsVersion,
  ) -> Result<IsolatedDtsTransformOutput> {
    self.run(|| {
      let comments = SingleThreadedComments::default();
      let fm = self.cm.new_source_file(filename.clone(), source);
      let unresolved_mark = Mark::new();
      let top_level_mark = Mark::new();
      let is_typescript = syntax.typescript();
      let mut program = try_with_handler(self.cm.clone(), Default::default(), |handler| {
        let mut had_error = false;
        let mut errors = vec![];
        let program = parse_file_as_program(&fm, syntax, target, Some(&comments), &mut errors);

        for error in errors {
          emit_syntax_error(error, handler);
          had_error = true;
        }

        let program = program.map_err(|error| {
          emit_syntax_error(error, handler);
          anyhow::Error::msg(SWC_SYNTAX_ERROR_CODE)
        })?;

        if had_error {
          bail!(SWC_SYNTAX_ERROR_CODE);
        }

        Ok(program)
      })
      .map_err(|error| {
        swc_diagnostics_to_rspack_error(error.diagnostics(), &self.cm)
          .unwrap_or_else(|| error.to_pretty_error().into())
      })?;
      program.visit_mut_with(&mut resolver(
        unresolved_mark,
        top_level_mark,
        is_typescript,
      ));

      self.emit_isolated_dts(&program, filename, unresolved_mark, target, &comments)
    })
  }
}

#[derive(Default)]
struct DtsReferenceCollector {
  references: Vec<String>,
  seen: HashSet<String>,
}

impl DtsReferenceCollector {
  fn push(&mut self, value: String) {
    if self.seen.insert(value.clone()) {
      self.references.push(value);
    }
  }
}

impl Visit for DtsReferenceCollector {
  fn visit_import_decl(&mut self, node: &ImportDecl) {
    // Matches import declarations, for example:
    //   import type { Foo } from "./foo";
    //   import "./foo";
    self.push(node.src.value.to_string_lossy().into_owned());
  }

  fn visit_export_all(&mut self, node: &ExportAll) {
    // Matches export-all declarations, for example:
    //   export * from "./foo";
    self.push(node.src.value.to_string_lossy().into_owned());
  }

  fn visit_named_export(&mut self, node: &NamedExport) {
    // Matches named re-exports, for example:
    //   export type { Foo } from "./foo";
    if let Some(src) = &node.src {
      self.push(src.value.to_string_lossy().into_owned());
    }
  }

  fn visit_ts_import_type(&mut self, node: &TsImportType) {
    // Matches inline import types, for example:
    //   export type Foo = import("./foo").Foo;
    self.push(node.arg.value.to_string_lossy().into_owned());
    node.visit_children_with(self);
  }

  fn visit_ts_external_module_ref(&mut self, node: &TsExternalModuleRef) {
    // Matches TypeScript import-equals declarations, for example:
    //   import Foo = require("./foo");
    self.push(node.expr.value.to_string_lossy().into_owned());
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
    input_source_map: Option<InputSourceMapResult>,
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
      source_map_kind,
      names: Default::default(),
    };

    let input_source_map = input_source_map.unwrap_or_else(|| {
      native_input_source_map(&self.fm.src, &self.fm.name, &built_input.input_source_map)
    })?;

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
      input_source_map,
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
      emit_syntax_error(e, handler);
      error = true;
    }

    let res = program_result.map_err(|e| {
      emit_syntax_error(e, handler);
      anyhow::Error::msg(SWC_SYNTAX_ERROR_CODE)
    });

    if error {
      return Err(anyhow::anyhow!(SWC_SYNTAX_ERROR_CODE));
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
            self
              .parse_js(
                self.fm.clone(),
                handler,
                target,
                syntax,
                is_module,
                Some(&self.comments).map(|c| c as &dyn Comments),
              )
              .map(|program| (program, false))
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
      .map_err(|error| {
        swc_diagnostics_to_rspack_error(error.diagnostics(), &self.cm)
          .unwrap_or_else(|| error.to_pretty_error().into())
      })
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
            let swc_core_version = rspack_workspace::rspack_swc_core_version!();
            let help_msg = formatdoc!{"
              The version of the SWC Wasm plugin you're using might not be compatible with `builtin:swc-loader`.
              The `swc_core` version of the current `rspack_core` is {swc_core_version}. 
              Please check the `swc_core` version of SWC Wasm plugin to make sure these versions are within the compatible range.
              See this guide as a reference for selecting SWC Wasm plugin versions: https://rspack.rs/errors/swc-plugin-version"};
            let mut error = swc_diagnostics_to_rspack_error(err.diagnostics(), &self.cm)
              .unwrap_or_else(|| rspack_error::error!(err.to_pretty_string()));
            error.help = Some(help_msg);
            error.code = Some(SWC_MIETTE_DIAGNOSTIC_CODE.into());
            error
          } else {
            let mut error = swc_diagnostics_to_rspack_error(err.diagnostics(), &self.cm)
              .unwrap_or_else(|| rspack_error::error!(err.to_pretty_string()));
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
