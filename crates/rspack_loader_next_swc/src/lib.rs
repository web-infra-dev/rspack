#![feature(let_chains)]

mod compiler;
mod options;
mod transformer;

use std::collections::HashMap;
use std::default::Default;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use compiler::{IntoJsAst, SwcCompiler};
use next_custom_transforms::chain_transforms::{custom_before_pass, TransformOptions};
use next_custom_transforms::transforms::cjs_optimizer::PackageConfig;
use next_custom_transforms::transforms::{
  cjs_optimizer, fonts, named_import_transform, optimize_server_react, react_server_components,
  server_actions,
};
use once_cell::sync::Lazy;
use options::NextSwcLoaderJsOptions;
use preset_env_base::version::Version;
use regex::Regex;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{Mode, RunnerContext};
use rspack_error::{error, AnyhowError, Result};
use rspack_loader_runner::{Identifiable, Identifier, Loader, LoaderContext};
use rspack_paths::Utf8PathBuf;
use rspack_plugin_javascript::ast::{self, SourceMapConfig};
use rspack_plugin_javascript::TransformOutput;
use rustc_hash::FxHashMap;
use sugar_path::SugarPath;
use swc::config::{
  GlobalInliningPassEnvs, GlobalPassOption, JscConfig, JscExperimental, ModuleConfig,
  OptimizerConfig, SimplifyOption, TransformConfig,
};
use swc_config::config_types::MergingOption;
use swc_core::base::config::SourceMapsConfig;
use swc_core::base::config::{InputSourceMap, OutputCharset};
use swc_core::common::collections::AHashMap;
use swc_core::ecma::atoms::Atom;
use swc_core::ecma::parser::{EsSyntax, Syntax, TsSyntax};
use swc_core::ecma::transforms::compat::es2015::regenerator;
use swc_core::ecma::visit::VisitWith;
use transformer::IdentCollector;

static NODE_MODULES_PATH: Lazy<Regex> = Lazy::new(|| Regex::new("[\\/]node_modules[\\/]").unwrap());

static EXCLUDED_PATHS: Lazy<Regex> = Lazy::new(|| {
  Regex::new("[\\\\/](cache[\\\\/][^\\/]+\\.zip[\\\\/]node_modules|__virtual__)[\\\\/]").unwrap()
});

static BABEL_INCLUDE_REGEXES: Lazy<Vec<Regex>> = Lazy::new(|| {
  vec![
    Regex::new(r"next[\\/]dist[\\/](esm[\\/])?shared[\\/]lib").unwrap(),
    Regex::new(r"next[\\/]dist[\\/](esm[\\/])?client").unwrap(),
    Regex::new(r"next[\\/]dist[\\/](esm[\\/])?pages").unwrap(),
    Regex::new(r"[\\/](strip-ansi|ansi-regex|styled-jsx)[\\/]").unwrap(),
  ]
});

static CWD: Lazy<PathBuf> = Lazy::new(|| ::std::env::current_dir().unwrap());

// these are exact code conditions checked
// for to force transpiling a `node_module`
static FORCE_TRANSPILE_CONDITIONS: Lazy<Regex> =
  Lazy::new(|| Regex::new("(next\\/font|next\\/dynamic|use server|use client)").unwrap());

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
    ext == ".cjs"
  } else {
    false
  }
}

fn should_output_common_js(filename: &Path) -> bool {
  is_common_js_file(filename)
}

fn is_resource_in_packages(resource: &str, package_names: &[String]) -> bool {
  package_names.iter().any(|p| {
    let node_modules_path =
      PathBuf::from("node_modules").join(p.replace("/", &std::path::MAIN_SEPARATOR.to_string()));
    resource.contains(&format!(
      "{}{}{}",
      std::path::MAIN_SEPARATOR,
      node_modules_path.to_string_lossy(),
      std::path::MAIN_SEPARATOR
    ))
  })
}

fn may_be_exclude(exclude_path: &str, transpile_packages: &[String]) -> bool {
  if BABEL_INCLUDE_REGEXES
    .iter()
    .any(|r| r.is_match(exclude_path))
  {
    return false;
  }

  let should_be_bundled = is_resource_in_packages(exclude_path, transpile_packages);
  if should_be_bundled {
    return false;
  }

  exclude_path.contains("node_modules")
}

fn get_transform_options(
  filename: &Utf8PathBuf,
  mode: Mode,
  options: &NextSwcLoaderJsOptions,
) -> TransformOptions {
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
    transpile_packages,
    modularize_imports,
    decorators,
    emit_decorator_metadata,
    regenerator_runtime_path,
    ..
  } = options;

  let pages_dir = pages_dir.clone().map(Utf8PathBuf::from);

  let is_page_file = match &pages_dir {
    Some(pages_dir) => filename.starts_with(pages_dir),
    None => false,
  };

  let disable_next_ssg = if *is_server { true } else { !is_page_file };

  let is_node_modules = NODE_MODULES_PATH.is_match(filename.as_str());
  let is_app_browser_layer = bundle_layer.as_deref() == Some("app-pages-browser");

  let disable_page_config = if *is_server {
    true
  } else {
    is_app_browser_layer && is_node_modules
  };

  let pages_dir = pages_dir.map(PathBuf::from);
  let app_dir = app_dir.clone().map(PathBuf::from);

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
    Some(swc_core::ecma::preset_env::Config {
      targets: Some(swc_core::ecma::preset_env::Targets::Versions(
        preset_env_base::BrowserData {
          node: Some(Version::from_str("18.20.4").unwrap()),
          ..Default::default()
        },
      )),
      path: CWD.clone(),
      ..Default::default()
    })
  } else {
    if supported_browsers.is_empty() {
      Some(Default::default())
    } else {
      Some(swc_core::ecma::preset_env::Config {
        targets: Some(swc_core::ecma::preset_env::Targets::Query(
          supported_browsers.clone().into(),
        )),
        ..Default::default()
      })
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
      decorators: *decorators,
      ..Default::default()
    }))
  } else {
    Some(Syntax::Es(EsSyntax {
      jsx: true,
      decorators: *decorators,
      import_attributes: true,
      ..Default::default()
    }))
  };

  let module = if should_output_common_js(filename.as_std_path()) {
    Some(ModuleConfig::CommonJs(Default::default()))
  } else {
    None
  };

  let mut typeofs: AHashMap<_, _> = Default::default();
  if *is_server {
    typeofs.insert("window".into(), "undefined".into());
  } else {
    typeofs.insert("window".into(), "object".into());
  }

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
        external_helpers: true.into(),
        transform: MergingOption::from(Some(TransformConfig {
          react: swc_core::ecma::transforms::react::Options {
            runtime: Some(swc_core::ecma::transforms::react::Runtime::Automatic),
            import_source: Some("react".to_string()),
            pragma_frag: Some("React.Fragment".to_string()),
            throw_if_namespace: Some(true),
            development: Some(is_development),
            refresh: if *has_react_refresh {
              Some(Default::default())
            } else {
              None
            },
            use_builtins: Some(true),
            ..Default::default()
          },
          optimizer: Some(OptimizerConfig {
            globals: Some(GlobalPassOption {
              envs: GlobalInliningPassEnvs::Map(HashMap::from_iter(vec![(
                "NODE_ENV".into(),
                if is_development {
                  "\"development\"".into()
                } else {
                  "\"production\"".into()
                },
              )])),
              typeofs,
              ..Default::default()
            }),
            simplify: Some(SimplifyOption::Bool(false)),
            ..Default::default()
          }),
          legacy_decorator: (*decorators).into(),
          decorator_metadata: (*emit_decorator_metadata).into(),
          regenerator: regenerator::Config {
            import_path: regenerator_runtime_path
              .as_ref()
              .map(|path| path.to_string().into()),
          },
          ..Default::default()
        })),
        ..Default::default()
      },
      module,
      // input_source_map,
      // source_maps,
      // inline_sources_content,
      // emit_source_map_columns,
      ..Default::default()
    },
    cwd: CWD.clone(),
    filename: filename.to_string(),
    ..Default::default()
  };

  let server_components = server_components.map(|_| {
    react_server_components::Config::WithOptions(react_server_components::Options {
      is_react_server_layer,
      dynamic_io_enabled: false,
    })
  });

  let modularize_imports =
    modularize_imports
      .as_ref()
      .map(|modularize_imports| modularize_imports::Config {
        packages: modularize_imports
          .clone()
          .into_iter()
          .map(|(key, value)| {
            (
              key,
              modularize_imports::PackageConfig {
                transform: match value.transform {
                  options::Transform::String(s) => modularize_imports::Transform::String(s),
                  options::Transform::Vec(vec) => modularize_imports::Transform::Vec(vec),
                },
                prevent_full_import: value.prevent_full_import,
                handle_default_import: value.handle_default_import,
                handle_namespace_import: value.handle_namespace_import,
                skip_default_conversion: value.skip_default_conversion,
              },
            )
          })
          .collect::<HashMap<_, _>>(),
      });

  let optimize_server_react = if *optimize_server_react {
    Some(optimize_server_react::Config {
      optimize_use_state: false,
    })
  } else {
    None
  };

  TransformOptions {
    swc: swc_options.clone(),
    disable_next_ssg,
    disable_page_config,
    pages_dir,
    app_dir,
    is_page_file,
    is_development,
    is_server_compiler: *is_server,
    prefer_esm: *esm,
    server_components,
    styled_jsx: Default::default(),
    styled_components,
    remove_console: None,
    react_remove_properties: None,
    relay: None,
    shake_exports: None,
    emotion,
    modularize_imports,
    auto_modularize_imports: None,
    optimize_barrel_exports: None,
    font_loaders: Some(fonts::Config {
      font_loaders: vec!["next/font/local".into(), "next/font/google".into()],
      relative_file_path_from_root,
    }),
    server_actions,
    cjs_require_optimizer,
    optimize_server_react,
    debug_function_name: is_development,
    lint_codemod_comments: true,
  }
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

    let filename = resource_path.clone();

    // Ensure `.d.ts` are not processed.
    if filename.as_str().ends_with(".d.ts") {
      return Ok(());
    }

    let Some(content) = loader_context.take_content() else {
      return Ok(());
    };
    let source = content.into_string_lossy();

    let should_maybe_exclude =
      may_be_exclude(resource_path.as_str(), &self.options.transpile_packages);
    if should_maybe_exclude {
      if !FORCE_TRANSPILE_CONDITIONS.is_match(&source) {
        let map = loader_context.take_source_map();
        loader_context.finish_with((source, map));
        return Ok(());
      }
    }

    let mut opts = get_transform_options(
      &filename,
      loader_context.context.options.mode,
      &self.options,
    );

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

    opts.swc.source_maps = source_maps.clone();
    opts.swc.source_file_name = Some(filename.to_string());
    opts.swc.config.input_source_map = input_source_map;
    opts.swc.config.inline_sources_content = inline_sources_content;
    opts.swc.config.emit_source_map_columns = emit_source_map_columns;

    let c = SwcCompiler::new(resource_path.into_std_path_buf(), source, opts.swc.clone())
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

  async fn pitch(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    if let Some(resource_path) = loader_context.resource_path() {
      let NextSwcLoaderJsOptions {
        transpile_packages,
        pnp,
        ..
      } = &self.options;

      let should_maybe_exclude = may_be_exclude(resource_path.as_str(), transpile_packages);

      if
      // if it might be excluded/no-op we can't use pitch loader
      !should_maybe_exclude &&
        // TODO: investigate swc file reading in PnP mode?
        !pnp &&
        !EXCLUDED_PATHS.is_match(resource_path.as_str()) &&
      loader_context.loader_items.len() as i32 - 1 == loader_context.loader_index &&
      resource_path.is_absolute()
      // !(await isWasm())
      {
        loader_context
          .file_dependencies
          .insert(resource_path.as_std_path().to_path_buf());
        return self.loader_impl(loader_context);
      }
    }

    Ok(())
  }
}

impl Identifiable for NextSwcLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}

#[cfg(test)]
mod tests {
  use std::{collections::HashMap, str::FromStr};

  use rspack_core::Mode;
  use rspack_paths::Utf8PathBuf;

  use crate::{
    get_transform_options,
    options::{NextSwcLoaderJsOptions, PackageConfig, Transform},
  };

  #[test]
  fn test_get_transform_options() {
    let input = NextSwcLoaderJsOptions {
      root_dir: "/test/e2e/app-dir/app".to_string(),
      is_server: true,
      pages_dir: Some("/test/e2e/app-dir/app/pages".to_string()),
      app_dir: Some("/test/e2e/app-dir/app/app".to_string()),
      has_react_refresh: false,
      optimize_server_react: true,
      supported_browsers: vec![
        "chrome 64".to_string(),
        "edge 79".to_string(),
        "firefox 67".to_string(),
        "opera 51".to_string(),
        "safari 12".to_string(),
      ],
      swc_cache_dir: "/test/e2e/app-dir/app/.next/cache/swc".to_string(),
      server_components: Some(true),
      server_reference_hash_salt: "J6craDGodsVA0OsOU/auvoNP8Gqeux/F8i6gTX9XajA=".to_string(),
      bundle_layer: Some("rsc".to_string()),
      esm: true,
      transpile_packages: vec!["geist".to_string(), "lucide-react".to_string()],
      pnp: false,
      modularize_imports: Some(HashMap::from_iter(vec![(
        "@mui/icons-material".to_string(),
        PackageConfig {
          transform: Transform::String("@mui/icons-material/{{member}}".to_string()),
          prevent_full_import: false,
          handle_default_import: false,
          handle_namespace_import: false,
          skip_default_conversion: false,
        },
      )])),
      decorators: false,
      emit_decorator_metadata: false,
      regenerator_runtime_path: Some("next/dist/compiled/regenerator-runtime".to_string()),
    };

    let filename = Utf8PathBuf::from_str("/test/e2e/app-dir/app/middleware.js").unwrap();
    let output = get_transform_options(&filename, Mode::Production, &input);

    println!("{:#?}", output);
    // disable_next_ssg
    // disable_page_config
    // pages_dir
    // app_dir
    // is_page_file
    // is_development
    // is_server_compiler
    // prefer_esm
    // server_components

    // TODO: styled_jsx

    // modularize_imports
    // auto_modularize_imports
  }
}
