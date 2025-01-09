#![feature(let_chains)]

mod compiler;
mod options;
mod transformer;

use std::default::Default;
use std::path::{Path, PathBuf};

use compiler::{IntoJsAst, SwcCompiler};
use next_custom_transforms::chain_transforms::{custom_before_pass, TransformOptions};
use next_custom_transforms::transforms::cjs_optimizer::PackageConfig;
use next_custom_transforms::transforms::{
  cjs_optimizer, fonts, react_server_components, server_actions,
};
use once_cell::sync::Lazy;
use options::NextSwcLoaderJsOptions;
pub use options::SwcLoaderJsOptions;
use regex::Regex;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::RunnerContext;
use rspack_error::{error, AnyhowError, Result};
use rspack_loader_runner::{Identifiable, Identifier, Loader, LoaderContext};
use rspack_paths::Utf8PathBuf;
use rspack_plugin_javascript::ast::{self, SourceMapConfig};
use rspack_plugin_javascript::TransformOutput;
use rustc_hash::FxHashMap;
use sugar_path::SugarPath;
use swc::config::{JscConfig, JscExperimental, ModuleConfig};
use swc_core::base::config::SourceMapsConfig;
use swc_core::base::config::{InputSourceMap, OutputCharset};
use swc_core::ecma::atoms::Atom;
use swc_core::ecma::parser::{EsSyntax, Syntax, TsSyntax};
use swc_core::ecma::visit::VisitWith;
use transformer::IdentCollector;

static NODE_MODULES_PATH: Lazy<Regex> = Lazy::new(|| Regex::new("[\\/]node_modules[\\/]").unwrap());

fn is_type_script_file(path: &Path) -> bool {
  if let Some(extension) = path.extension() {
    let ext = extension.to_string_lossy().to_lowercase();
    ext == "ts" || ext == "tsx"
  } else {
    false
  }
}

fn is_common_js_file(path: &Path) -> bool {
  if let Some(extension) = path.extension() {
    let ext = extension.to_string_lossy().to_lowercase();
    ext == "ts"
  } else {
    false
  }
}

fn should_output_common_js(filename: &Path) -> bool {
  is_common_js_file(filename)
}

#[cacheable]
#[derive(Debug)]
pub struct NextSwcLoader {
  identifier: Identifier,
  options: NextSwcLoaderJsOptions,
}

impl NextSwcLoader {
  pub fn new(raw_options: &str) -> Result<Self, serde_json::Error> {
    Ok(Self {
      identifier: NEXT_SWC_LOADER_IDENTIFIER.into(),
      options: raw_options.try_into()?,
    })
  }

  /// Panics:
  /// Panics if `identifier` passed in is not starting with `builtin:swc-loader`.
  pub fn with_identifier(mut self, identifier: Identifier) -> Self {
    assert!(identifier.starts_with(NEXT_SWC_LOADER_IDENTIFIER));
    self.identifier = identifier;
    self
  }

  fn loader_impl(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let resource_path = loader_context
      .resource_path()
      .map(|p| p.to_path_buf())
      .unwrap_or_default();
    let Some(content) = loader_context.take_content() else {
      return Ok(());
    };

    let source = content.into_string_lossy();

    let NextSwcLoaderJsOptions {
      root_dir,
      pages_dir,
      is_server,
      bundle_layer,
      app_dir,
      esm,
      server_components,
      server_reference_hash_salt,
      has_react_refresh,
      optimize_server_react,
      supported_browsers,
      swc_cache_dir,
    } = &self.options;

    let pages_dir = pages_dir.clone().map(Utf8PathBuf::from);
    let filename = resource_path.clone();

    let is_page_file = match &pages_dir {
      Some(pages_dir) => filename.starts_with(pages_dir),
      None => false,
    };

    let disable_next_ssg = if *is_server { true } else { is_page_file };

    let is_node_modules = NODE_MODULES_PATH.is_match(filename.as_str());
    let is_app_browser_layer = bundle_layer.as_deref() == Some("app-pages-browser");

    let disable_page_config = if *is_server {
      true
    } else {
      is_app_browser_layer && is_node_modules
    };

    let pages_dir = pages_dir.map(PathBuf::from);
    let app_dir = app_dir.clone().map(PathBuf::from);

    let mode = loader_context.context.options.mode;
    let is_development = mode.is_development();

    let is_react_server_layer = matches!(
      bundle_layer.as_deref(),
      Some("rsc") | Some("action-browser") | Some("middleware") | Some("instrument")
    );

    let styled_components = if !is_react_server_layer {
      Some(Default::default())
    } else {
      None
    };

    let emotion = if !is_react_server_layer {
      Some(Default::default())
    } else {
      None
    };

    let root_dir = PathBuf::from(root_dir);
    let relative_file_path_from_root = filename
      .as_std_path()
      .relative(root_dir)
      .to_string_lossy()
      .to_string()
      .into();

    let is_app_router_pages_layer = matches!(
      bundle_layer.as_deref(),
      Some("rsc") | Some("ssr") | Some("action-browser") | Some("app-pages-browser")
    );
    let server_actions = if is_app_router_pages_layer {
      Some(server_actions::Config {
        is_react_server_layer,
        dynamic_io_enabled: false,
        hash_salt: server_reference_hash_salt.to_string(),
        cache_kinds: Default::default(),
      })
    } else {
      None
    };

    let mut packages: FxHashMap<String, PackageConfig> = Default::default();
    let mut transforms: FxHashMap<Atom, Atom> = Default::default();
    transforms.insert(
      "NextRequest".into(),
      "next/dist/server/web/spec-extension/request".into(),
    );
    transforms.insert(
      "NextResponse".into(),
      "next/dist/server/web/spec-extension/response".into(),
    );
    transforms.insert(
      "ImageResponse".into(),
      "next/dist/server/web/spec-extension/image-response".into(),
    );
    transforms.insert(
      "userAgentFromString".into(),
      "next/dist/server/web/spec-extension/user-agent".into(),
    );
    transforms.insert(
      "userAgent".into(),
      "next/dist/server/web/spec-extension/user-agent".into(),
    );
    let package_config = PackageConfig { transforms };
    packages.insert("next/server".to_string(), package_config);
    let cjs_require_optimizer = Some(cjs_optimizer::Config { packages });

    let env = if *is_server {
      None
    } else {
      if supported_browsers.is_empty() {
        Some(swc_core::ecma::preset_env::Config {
          targets: Some(swc_core::ecma::preset_env::Targets::Query(
            supported_browsers.clone().into(),
          )),
          ..Default::default()
        })
      } else {
        None
      }
    };

    let is_ts_file = if let Some(extension) = filename.as_std_path().extension() {
      let ext = extension.to_string_lossy().to_lowercase();
      ext == "ts"
    } else {
      false
    };
    let has_ts_syntax = is_type_script_file(&filename.as_std_path());
    let syntax = if has_ts_syntax {
      Some(Syntax::Typescript(TsSyntax {
        tsx: !is_ts_file,
        decorators: true,
        ..Default::default()
      }))
    } else {
      Some(Syntax::Es(EsSyntax {
        jsx: true,
        decorators: true,
        ..Default::default()
      }))
    };

    let module = if should_output_common_js(filename.as_std_path()) {
      Some(ModuleConfig::CommonJs(Default::default()))
    } else {
      None
    };

    let input_source_map = loader_context
      .source_map()
      .map(|source_map| InputSourceMap::Str(source_map.clone().to_json().unwrap()));

    let source_map_kind = loader_context.context.module_source_map_kind;

    let source_maps = if source_map_kind.enabled() {
      Some(SourceMapsConfig::Str("inline".to_string()))
    } else {
      None
    };

    let inline_sources_content = loader_context
      .context
      .module_source_map_kind
      .enabled()
      .into();

    let emit_source_map_columns = (!source_map_kind.cheap()).into();

    let swc_options = swc::config::Options {
      config: swc::config::Config {
        env,
        jsc: JscConfig {
          syntax,
          experimental: JscExperimental {
            keep_import_attributes: true.into(),
            emit_assert_for_import_attributes: true.into(),
            cache_root: Some(swc_cache_dir.to_string()),
            ..Default::default()
          },
          ..Default::default()
        },
        module,
        input_source_map,
        source_maps,
        inline_sources_content,
        emit_source_map_columns,
        ..Default::default()
      },
      ..Default::default()
    };

    let opts = TransformOptions {
      swc: swc_options.clone(),
      disable_next_ssg,
      disable_page_config,
      pages_dir,
      app_dir,
      is_page_file,
      is_development,
      is_server_compiler: *is_server,
      prefer_esm: *esm,
      server_components: server_components.map(react_server_components::Config::All),
      styled_jsx: Default::default(),
      styled_components,
      remove_console: None,
      react_remove_properties: None,
      relay: None,
      shake_exports: None,
      emotion,
      modularize_imports: None,
      auto_modularize_imports: None,
      optimize_barrel_exports: None,
      font_loaders: Some(fonts::Config {
        font_loaders: vec!["next/font/local".into(), "next/font/google".into()],
        relative_file_path_from_root,
      }),
      server_actions,
      cjs_require_optimizer,
      optimize_server_react: None,
      debug_function_name: is_development,
      lint_codemod_comments: true,
    };

    let c = SwcCompiler::new(resource_path.into_std_path_buf(), source, swc_options)
      .map_err(AnyhowError::from)?;

    let c_ref = &c;

    let built = c
      .parse(None, |_| {
        custom_before_pass(
          c_ref.cm.clone(),
          c_ref.fm.clone(),
          &opts,
          c_ref.comments.clone(),
          Default::default(),
          c_ref.unresolved_mark,
        )
      })
      .map_err(AnyhowError::from)?;

    let input_source_map = c
      .input_source_map(&built.input_source_map)
      .map_err(|e| error!(e.to_string()))?;
    let mut codegen_options = ast::CodegenOptions {
      target: Some(built.target),
      minify: Some(built.minify),
      input_source_map: input_source_map.as_ref(),
      ascii_only: built
        .output
        .charset
        .as_ref()
        .map(|v| matches!(v, OutputCharset::Ascii)),
      source_map_config: SourceMapConfig {
        enable: source_map_kind.source_map(),
        inline_sources_content: source_map_kind.source_map(),
        emit_columns: !source_map_kind.cheap(),
        names: Default::default(),
      },
      inline_script: Some(false),
      keep_comments: Some(true),
    };

    let program = c.transform(built)?;
    if source_map_kind.enabled() {
      let mut v = IdentCollector {
        names: Default::default(),
      };
      program.visit_with(&mut v);
      codegen_options.source_map_config.names = v.names;
    }
    let ast = c.into_js_ast(program);
    let TransformOutput { code, map } = ast::stringify(&ast, codegen_options)?;
    loader_context.finish_with((code, map));

    Ok(())
  }
}

pub const NEXT_SWC_LOADER_IDENTIFIER: &str = "builtin:next-swc-loader";

#[cacheable_dyn]
#[async_trait::async_trait]
impl Loader<RunnerContext> for NextSwcLoader {
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    #[allow(unused_mut)]
    let mut inner = || self.loader_impl(loader_context);
    #[cfg(debug_assertions)]
    {
      // Adjust stack to avoid stack overflow.
      stacker::maybe_grow(
        2 * 1024 * 1024, /* 2mb */
        4 * 1024 * 1024, /* 4mb */
        inner,
      )
    }
    #[cfg(not(debug_assertions))]
    inner()
  }
}

impl Identifiable for NextSwcLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}
