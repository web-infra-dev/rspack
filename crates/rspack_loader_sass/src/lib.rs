#![feature(let_chains)]

use std::{
  env,
  iter::Peekable,
  path::{Path, PathBuf},
  sync::Arc,
};

use crossbeam_channel::{unbounded, Sender};
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  rspack_sources::SourceMap, CompilationContext, CompilerContext, DependencyCategory,
  DependencyType, Resolve, ResolveOptionsWithDependencyType, ResolveResult, Resolver,
  ResolverFactory,
};
use rspack_error::{
  internal_error, Diagnostic, DiagnosticKind, Error, InternalError, Result, Severity,
  TraceableError,
};
use rspack_loader_runner::{Loader, LoaderContext};
use sass_embedded::{
  legacy::{
    IndentType, LegacyImporter, LegacyImporterResult, LegacyImporterThis, LegacyOptions,
    LegacyOptionsBuilder, LineFeed, OutputStyle,
  },
  Exception, Logger, Sass, SourceSpan, Url,
};
use serde::Deserialize;
use str_indices::utf16;

static IS_SPECIAL_MODULE_IMPORT: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^~[^/]+$").expect("TODO:"));
static IS_NATIVE_WIN32_PATH: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"(?i)^[a-z]:[/\\]|^\\\\").expect("TODO:"));
static MODULE_REQUEST: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[^?]*~").expect("TODO:"));
static IS_MODULE_IMPORT: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"^~([^/]+|[^/]+/|@[^/]+[/][^/]+|@[^/]+/?|@[^/]+[/][^/]+/)$").expect("TODO:")
});

fn get_os() -> &'static str {
  match env::consts::OS {
    "linux" => "linux",
    "macos" => "darwin",
    "windows" => "win32",
    os => panic!("dart-sass-embedded is not supported for {os}"),
  }
}

fn get_arch() -> &'static str {
  match env::consts::ARCH {
    "x86" => "ia32",
    "x86_64" => "x64",
    "aarch64" => "arm64",
    arch => panic!("dart-sass-embedded is not supported for {arch}"),
  }
}

fn dev_exe_path() -> PathBuf {
  let os = get_os();
  let arch = get_arch();
  PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR")))
    .join(format!("../../node_modules/sass-embedded-{os}-{arch}"))
    .join("dart-sass-embedded/dart-sass-embedded")
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SassLoaderOptions {
  sass_options: SassOptions,
  // `None` means open or close source map depends on whether in production mode.
  source_map: Option<bool>,
  additional_data: Option<String>,
  rspack_importer: bool,
  #[serde(rename = "__exePath")]
  __exe_path: PathBuf,
}

impl Default for SassLoaderOptions {
  fn default() -> Self {
    Self {
      rspack_importer: true,
      source_map: Default::default(),
      additional_data: Default::default(),
      sass_options: Default::default(),
      __exe_path: dev_exe_path(),
    }
  }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SassOptions {
  indented_syntax: Option<bool>,
  include_paths: Vec<PathBuf>,
  charset: Option<bool>,
  indent_type: Option<IndentType>,
  indent_width: Option<usize>,
  linefeed: Option<LineFeed>,
  output_style: Option<OutputStyle>,
  quiet_deps: Option<bool>,
  verbose: Option<bool>,
}

#[derive(Debug)]
struct RspackImporter {
  include_paths: Vec<PathBuf>,
  sass_module_resolve: Arc<Resolver>,
  sass_import_resolve: Arc<Resolver>,
  rspack_module_resolve: Arc<Resolver>,
  rspack_import_resolve: Arc<Resolver>,
}

impl RspackImporter {
  pub fn new(include_paths: Vec<PathBuf>, factory: Arc<ResolverFactory>) -> Self {
    let sass_module_resolve = factory.get(ResolveOptionsWithDependencyType {
      resolve_options: Some(Resolve {
        extensions: Some(vec![
          ".sass".to_owned(),
          ".scss".to_owned(),
          ".css".to_owned(),
        ]),
        alias: Some(Vec::new()),
        prefer_relative: Some(true),
        main_files: Some(vec!["_index".to_owned(), "index".to_owned()]),
        main_fields: Some(Vec::new()),
        // TODO: add restrictions field when resolver supports it.
        ..Default::default()
      }),
      resolve_to_context: false,
      dependency_type: DependencyType::Unknown,
      dependency_category: DependencyCategory::Unknown,
    });
    let sass_import_resolve = factory.get(ResolveOptionsWithDependencyType {
      resolve_options: Some(Resolve {
        extensions: Some(vec![
          ".sass".to_owned(),
          ".scss".to_owned(),
          ".css".to_owned(),
        ]),
        alias: Some(Vec::new()),
        prefer_relative: Some(true),
        main_files: Some(vec![
          "_index.import".to_owned(),
          "_index".to_owned(),
          "index.import".to_owned(),
          "index".to_owned(),
        ]),
        main_fields: Some(Vec::new()),
        ..Default::default()
      }),
      resolve_to_context: false,
      dependency_type: DependencyType::Unknown,
      dependency_category: DependencyCategory::Unknown,
    });
    let rspack_module_resolve = factory.get(ResolveOptionsWithDependencyType {
      resolve_options: Some(Resolve {
        // TODO: add dependencyType.
        condition_names: Some(vec!["sass".to_owned(), "style".to_owned()]),
        main_fields: Some(vec![
          "sass".to_owned(),
          "style".to_owned(),
          "main".to_owned(),
          "...".to_owned(),
        ]),
        main_files: Some(vec![
          "_index".to_owned(),
          "index".to_owned(),
          "...".to_owned(),
        ]),
        extensions: Some(vec![
          ".sass".to_owned(),
          ".scss".to_owned(),
          ".css".to_owned(),
        ]),
        prefer_relative: Some(true),
        ..Default::default()
      }),
      resolve_to_context: false,
      dependency_type: DependencyType::Unknown,
      dependency_category: DependencyCategory::Unknown,
    });
    let rspack_import_resolve = factory.get(ResolveOptionsWithDependencyType {
      resolve_options: Some(Resolve {
        condition_names: Some(vec!["sass".to_owned(), "style".to_owned()]),
        main_fields: Some(vec![
          "sass".to_owned(),
          "style".to_owned(),
          "main".to_owned(),
          "...".to_owned(),
        ]),
        main_files: Some(vec![
          "_index.import".to_owned(),
          "_index".to_owned(),
          "index.import".to_owned(),
          "index".to_owned(),
          "...".to_owned(),
        ]),
        extensions: Some(vec![
          ".sass".to_owned(),
          ".scss".to_owned(),
          ".css".to_owned(),
        ]),
        prefer_relative: Some(true),
        ..Default::default()
      }),
      resolve_to_context: false,
      dependency_type: DependencyType::Unknown,
      dependency_category: DependencyCategory::Unknown,
    });
    Self {
      include_paths,
      sass_import_resolve,
      sass_module_resolve,
      rspack_import_resolve,
      rspack_module_resolve,
    }
  }
}

#[allow(clippy::result_large_err)]
fn get_possible_requests(
  url: &str,
  for_rspack_resolver: bool,
  from_import: bool,
) -> std::result::Result<Vec<String>, Exception> {
  let mut request = url.to_string();
  if for_rspack_resolver {
    if MODULE_REQUEST.is_match(url) {
      request = MODULE_REQUEST.replace(&request, "").to_string();
    }
    if IS_MODULE_IMPORT.is_match(url) {
      if !request.ends_with('/') {
        request.push('/');
      }
      if request == url {
        return Ok(vec![request]);
      }
      return Ok(vec![request, url.to_string()]);
    }
  }

  let request_path = Path::new(&request);
  let ext = match request_path.extension() {
    Some(ext) if ext.to_string_lossy() == "css" => return Ok(Vec::new()),
    Some(ext) => format!(".{}", ext.to_string_lossy()),
    None => "".to_owned(),
  };

  let dirname = request_path.parent();
  let dirname = if matches!(dirname, None)
    || matches!(
      dirname,
      Some(p) if p == Path::new("") || p == Path::new(".")
    ) {
    "".to_owned()
  } else {
    // SAFETY: `unwrap` is ok since `None` is checked in if branch.
    format!("{}/", dirname.expect("TODO:").display())
  };
  let basename = request_path
    .file_name()
    .ok_or_else(|| Exception::new("The path of sass's dependency should have file name"))?
    .to_string_lossy();
  // SAFETY: `unwrap` is ok since `request_path` has file name is checked before.
  let basename_without_ext = request_path.file_stem().expect("TODO:").to_string_lossy();

  let mut requests = Vec::new();
  if from_import {
    requests.push(format!("{dirname}_{basename_without_ext}.import{ext}"));
    requests.push(format!("{dirname}{basename_without_ext}.import{ext}"));
  }
  requests.push(format!("{dirname}_{basename}"));
  requests.push(format!("{dirname}{basename}"));
  if for_rspack_resolver {
    requests.push(url.to_string());
  }
  Ok(requests.into_iter().unique().collect())
}

#[derive(Debug)]
struct Resolution<'r, 'c, I: Iterator<Item = String>> {
  resolve: &'r Resolver,
  context: &'c Path,
  possible_requests: I,
}

fn start_resolving<'r, 'c, I: Iterator<Item = String>>(
  mut resolutions: Peekable<impl Iterator<Item = Resolution<'r, 'c, I>>>,
) -> Option<PathBuf> {
  let resolution = resolutions.peek_mut()?;
  if let Some(possible_request) = resolution.possible_requests.next() {
    if let Ok(ResolveResult::Resource(resource)) = resolution
      .resolve
      .resolve(resolution.context, &possible_request)
    {
      Some(resource.path)
    } else {
      start_resolving(resolutions)
    }
  } else {
    resolutions.next();
    start_resolving(resolutions)
  }
}

impl LegacyImporter for RspackImporter {
  fn call(
    &self,
    options: &LegacyImporterThis,
    request: &str,
    context: &str,
  ) -> sass_embedded::Result<Option<LegacyImporterResult>> {
    let from_importer = options.from_import;
    let need_emulate_sass_resolver = !IS_SPECIAL_MODULE_IMPORT.is_match(request)
      && !request.starts_with('/')
      && !IS_NATIVE_WIN32_PATH.is_match(request);

    let mut resolutions = Vec::new();
    if !self.include_paths.is_empty() && need_emulate_sass_resolver {
      let sass_possible_requests = get_possible_requests(request, false, from_importer)?;
      resolutions.extend(self.include_paths.iter().map(|context| Resolution {
        resolve: if from_importer {
          &self.sass_import_resolve
        } else {
          &self.sass_module_resolve
        },
        context,
        possible_requests: sass_possible_requests.clone().into_iter(),
      }));
    }

    let rspack_possible_requests = get_possible_requests(request, true, from_importer)?;
    resolutions.push(Resolution {
      resolve: if from_importer {
        &self.rspack_import_resolve
      } else {
        &self.rspack_module_resolve
      },
      context: Path::new(context)
        .parent()
        .ok_or_else(|| Exception::new(format!("dirname of {context} is `None`")))?,
      possible_requests: rspack_possible_requests.into_iter(),
    });
    Ok(start_resolving(resolutions.into_iter().peekable()).map(LegacyImporterResult::file))
  }
}

#[derive(Debug)]
struct RspackLogger {
  tx: Sender<Vec<Diagnostic>>,
}

impl Logger for RspackLogger {
  fn warn(&self, message: &str, options: &sass_embedded::LoggerWarnOptions) {
    self
      .tx
      .send(sass_log_to_diagnostics(
        Severity::Warn,
        message,
        options.span.as_ref(),
      ))
      .expect("TODO:");
  }
}

#[derive(Debug)]
pub struct SassLoader {
  options: SassLoaderOptions,
}

impl SassLoader {
  pub fn new(options: SassLoaderOptions) -> Self {
    Self { options }
  }

  fn get_sass_options(
    &self,
    loader_context: &LoaderContext<'_, '_, CompilerContext, CompilationContext>,
    content: String,
    logger: RspackLogger,
  ) -> LegacyOptions {
    let mut builder = LegacyOptionsBuilder::default()
      .data(
        if let Some(additional_data) = &self.options.additional_data {
          format!("{additional_data}\n{content}")
        } else {
          content
        },
      )
      .logger(logger)
      .file(loader_context.resource_path)
      .source_map(
        self
          .options
          .source_map
          .unwrap_or_else(|| loader_context.compiler_context.options.devtool.enabled()),
      )
      .source_map_contents(true)
      // TODO: use OutputStyle::Compressed when loader_context.mode is production.
      // .output_style(
      //   self
      //     .options
      //     .sass_options
      //     .output_style
      //     .unwrap_or(OutputStyle::Expanded),
      // )
      .indented_syntax(
        self
          .options
          .sass_options
          .indented_syntax
          .unwrap_or_else(|| {
            Path::new(loader_context.resource_path)
              .extension()
              .map(|ext| ext == "sass")
              .unwrap_or_default()
          }),
      );

    let mut include_paths = vec![env::current_dir().expect("TODO:")];
    include_paths.extend(self.options.sass_options.include_paths.iter().map(|path| {
      if path.is_absolute() {
        path.to_owned()
      } else {
        env::current_dir().expect("TODO:").join(path)
      }
    }));
    builder = builder.include_paths(&include_paths);

    if self.options.rspack_importer {
      builder = builder.importer(RspackImporter::new(
        include_paths,
        Arc::clone(&loader_context.compiler_context.resolver_factory),
      ));
    }

    if let Some(charset) = &self.options.sass_options.charset {
      builder = builder.charset(*charset);
    }
    if let Some(indent_type) = &self.options.sass_options.indent_type {
      builder = builder.indent_type(indent_type.to_owned());
    }
    if let Some(linefeed) = &self.options.sass_options.linefeed {
      builder = builder.linefeed(linefeed.to_owned());
    }
    if let Some(indent_width) = &self.options.sass_options.indent_width {
      builder = builder.indent_width(*indent_width);
    }
    if let Some(quiet_deps) = &self.options.sass_options.quiet_deps {
      builder = builder.quiet_deps(*quiet_deps);
    }
    if let Some(verbose) = &self.options.sass_options.verbose {
      builder = builder.verbose(*verbose);
    }

    builder.build()
  }
}

#[async_trait::async_trait]
impl Loader<CompilerContext, CompilationContext> for SassLoader {
  fn name(&self) -> &'static str {
    "builtin:sass-loader"
  }

  async fn run(
    &self,
    loader_context: &mut LoaderContext<'_, '_, CompilerContext, CompilationContext>,
  ) -> Result<()> {
    let content = loader_context.content.to_owned();
    let (tx, rx) = unbounded();
    let logger = RspackLogger { tx };
    let sass_options = self.get_sass_options(loader_context, content.try_into_string()?, logger);
    let result = Sass::new(&self.options.__exe_path)
      .map_err(|e| {
        rspack_error::Error::InternalError(InternalError {
          severity: Severity::Error,
          error_message: format!(
            "{}: The dart-sass-embedded path is {}, your OS is {}, your Arch is {}",
            e.message(),
            &self.options.__exe_path.display(),
            get_os(),
            get_arch(),
          ),
        })
      })?
      .render(sass_options)
      .map_err(sass_exception_to_error)?;
    let source_map = result
      .map
      .map(|map| -> Result<SourceMap> {
        let mut map = SourceMap::from_slice(&map).map_err(|e| internal_error!(e.to_string()))?;
        for source in map.sources_mut() {
          if source.starts_with("file:") {
            *source = Url::parse(source)
              .expect("TODO:")
              .to_file_path()
              .expect("TODO:")
              .display()
              .to_string();
          }
        }
        Ok(map)
      })
      .transpose()?;

    loader_context.content = result.css.into();
    loader_context.source_map = source_map;
    loader_context
      .diagnostic
      .append(&mut rx.into_iter().flatten().collect_vec());
    Ok(())
  }

  fn as_any(&self) -> &dyn std::any::Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
    self
  }
}

fn sass_exception_to_error(e: Exception) -> Error {
  if let Some(span) = e.span()
    && let Some(message) = e.sass_message()
    && let Some(e) = make_traceable_error("Sass Error", message, span) {
    Error::TraceableError(e.with_kind(DiagnosticKind::Scss))
  } else {
    internal_error!(e.message().to_string())
  }
}

fn sass_log_to_diagnostics(
  severity: Severity,
  message: &str,
  span: Option<&SourceSpan>,
) -> Vec<Diagnostic> {
  let title = match severity {
    Severity::Error => "Sass Error",
    Severity::Warn => "Sass Warning",
  };
  if let Some(span) = span && let Some(e) = make_traceable_error(title, message, span) {
    Error::TraceableError(e.with_kind(DiagnosticKind::Scss).with_severity(severity)).into()
  } else {
    let f = match severity {
      Severity::Error => Diagnostic::error,
      Severity::Warn => Diagnostic::warn,
    };
    vec![f(title.to_string(), message.to_string(), 0, 0).with_kind(DiagnosticKind::Scss)]
  }
}

fn make_traceable_error(title: &str, message: &str, span: &SourceSpan) -> Option<TraceableError> {
  span
    .url
    .as_ref()
    .map(|url| {
      url
        .to_file_path()
        .expect("TODO:")
        .to_string_lossy()
        .to_string()
    })
    .and_then(|path| {
      std::fs::read_to_string(&path)
        .ok()
        .map(|source| (path, source))
    })
    .map(|(path, source)| {
      let start = utf16::to_byte_idx(&source, span.start.offset);
      let end = utf16::to_byte_idx(&source, span.end.offset);
      TraceableError::from_file(
        path,
        source,
        start,
        end,
        title.to_string(),
        message.to_string(),
      )
    })
}
