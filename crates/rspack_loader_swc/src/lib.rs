mod collect_ts_info;
mod options;
mod plugin;
mod transformer;
mod transforms;

use std::{default::Default, path::Path, sync::Arc};

use options::SwcCompilerOptionsWithAdditional;
pub use options::SwcLoaderJsOptions;
pub use plugin::SwcLoaderPlugin;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  COLLECTED_TYPESCRIPT_INFO_PARSE_META_KEY, Mode, Module, RSCModuleType, RunnerContext,
};
use rspack_error::{Diagnostic, Error, Result};
use rspack_javascript_compiler::{JavaScriptCompiler, TransformOutput};
use rspack_loader_runner::{Identifier, Loader, LoaderContext};
#[cfg(allocative)]
use rspack_util::allocative;
pub use rspack_workspace::rspack_swc_core_version;
use sugar_path::SugarPath;
use swc_config::{merge::Merge, types::MergingOption};
use swc_core::{
  base::config::{InputSourceMap, TransformConfig},
  common::{FileName, SyntaxContext},
  ecma::ast::noop_pass,
};

use crate::collect_ts_info::collect_typescript_info;

#[cacheable]
#[derive(Debug)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct SwcLoader {
  identifier: Identifier,
  #[cfg_attr(allocative, allocative(skip))]
  options_with_additional: SwcCompilerOptionsWithAdditional,
}

impl SwcLoader {
  pub fn new(raw_options: &str) -> Result<Self, serde_json::Error> {
    Ok(Self {
      identifier: SWC_LOADER_IDENTIFIER.into(),
      options_with_additional: raw_options.try_into()?,
    })
  }

  /// Panics:
  /// Panics if `identifier` passed in is not starting with `builtin:swc-loader`.
  pub fn with_identifier(mut self, identifier: Identifier) -> Self {
    assert!(identifier.starts_with(SWC_LOADER_IDENTIFIER));
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

    let swc_options = {
      let mut swc_options = self.options_with_additional.swc_options.clone();
      if swc_options.config.jsc.transform.as_ref().is_some() {
        let mut transform = TransformConfig::default();
        transform.react.development =
          Some(Mode::is_development(&loader_context.context.options.mode));
        swc_options
          .config
          .jsc
          .transform
          .merge(MergingOption::from(Some(transform)));
      }

      if loader_context.context.source_map_kind.enabled() {
        if let Some(pre_source_map) = loader_context.source_map().cloned()
          && let Ok(source_map) = pre_source_map.to_json()
        {
          swc_options.config.input_source_map = Some(InputSourceMap::Str(source_map))
        }
      } else {
        swc_options.config.input_source_map = Some(InputSourceMap::Bool(false));
      }
      swc_options.filename = resource_path.as_str().to_string();
      swc_options.source_file_name = Some(resource_path.as_str().to_string());

      if swc_options.config.jsc.target.is_some() && swc_options.config.env.is_some() {
        loader_context.emit_diagnostic(Diagnostic::warn(
          SWC_LOADER_IDENTIFIER.to_string(),
          "`env` and `jsc.target` cannot be used together".to_string(),
        ));
      }

      #[cfg(feature = "plugin")]
      {
        swc_options.runtime_options =
          swc_options
            .runtime_options
            .plugin_runtime(std::sync::Arc::new(
              rspack_util::swc::runtime::WasmtimeRuntime,
            ));
      }

      swc_options
    };

    let javascript_compiler = JavaScriptCompiler::new();
    let filename = Arc::new(FileName::Real(resource_path.clone().into_std_path_buf()));

    let source = content.into_string_lossy();
    let is_typescript =
      matches!(swc_options.config.jsc.syntax, Some(syntax) if syntax.typescript());
    let mut collected_ts_info = None;

    let TransformOutput {
      code,
      mut map,
      diagnostics,
    } = javascript_compiler.transform(
      source,
      Some(filename.clone()),
      swc_options,
      Some(loader_context.context.source_map_kind),
      |program, unresolved_mark| {
        if !is_typescript {
          return;
        }
        let Some(options) = &self
          .options_with_additional
          .rspack_experiments
          .collect_typescript_info
        else {
          return;
        };
        collected_ts_info = Some(collect_typescript_info(
          program,
          SyntaxContext::empty().apply_mark(unresolved_mark),
          options,
        ));
      },
      |_| {
        (
          if self
            .options_with_additional
            .rspack_experiments
            .react_server_components
          {
            let module = &loader_context.context.module;

            // Avoid transforming the redirected server entry module to prevent duplicate RSC metadata generation.
            if loader_context
              .resource_query()
              .is_some_and(|q| q.contains("skip-rsc-transform"))
            {
              Box::new(noop_pass()) as Box<dyn swc_core::ecma::ast::Pass>
            } else {
              let is_react_server_layer = module
                .get_layer()
                .is_some_and(|layer| layer == "react-server-components");
              let build_info = loader_context.context.module.build_info_mut();
              Box::new(transforms::server_components(
                filename,
                transforms::Config::WithOptions(transforms::Options {
                  is_react_server_layer,
                }),
                &mut build_info.rsc,
              )) as Box<dyn swc_core::ecma::ast::Pass>
            }
          } else {
            Box::new(noop_pass()) as Box<dyn swc_core::ecma::ast::Pass>
          },
          transformer::transform(&self.options_with_additional.rspack_experiments),
        )
      },
    )?;

    let module = &loader_context.context.module;
    let is_react_server_layer = module
      .get_layer()
      .is_some_and(|layer| layer == "react-server-components");
    if is_react_server_layer && let Some(rsc) = module.build_info().rsc.as_ref() {
      let ids = rsc
        .client_refs
        .iter()
        .filter_map(|export| export.as_str())
        .collect::<Vec<_>>()
        .join(", ");

      if rsc.module_type == RSCModuleType::ServerEntry {
        if rsc
          .client_refs
          .iter()
          .any(|client_ref| client_ref.as_str() == Some("*"))
        {
          // TODO: remove panic
          panic!(
            r#"It's currently unsupported to use "export *" in a server entry. Please use named exports instead."#
          );
        }

        let mut esm_source = r#"import { createResourcesProxy } from "@rspack/rsc-runtime";
"#
        .to_string();

        for client_ref in &rsc.client_refs {
          match client_ref.as_str() {
            Some("default") => {
              // 增加 skip-rsc-transform 查询参数，避免代理模块中导入 server entry 模块，被重复生成为代理代码
              esm_source.push_str(&format!(
                r#"import _default from "{}?skip-rsc-transform";
export default createResourcesProxy(
_default,
"{}",
)
"#,
                loader_context.resource(),
                loader_context.resource()
              ));
            }
            Some(ident) => {
              esm_source.push_str(&format!(
                r#"import {{ {} as _original_{} }} from "{}?skip-rsc-transform";
export const {} = createResourcesProxy(
_original_{},
"{}",
)
"#,
                ident,
                ident,
                loader_context.resource(),
                ident,
                ident,
                loader_context.resource(),
              ));
            }
            _ => {}
          }
        }

        loader_context.finish_with(esm_source);
        return Ok(());
      }

      if rsc.module_type == RSCModuleType::Client {
        // TODO 生成代码需要区分 ESM 和 CJS
        let mut esm_source = format!(
          r#"import {{ registerClientReference }} from "react-server-dom-webpack/server"
"#,
        );

        if rsc
          .client_refs
          .iter()
          .any(|client_ref| client_ref.as_str() == Some("*"))
        {
          // TODO: remove panic
          panic!(
            r#"It's currently unsupported to use "export *" in a client boundary. Please use named exports instead."#
          );
        }

        for client_ref in &rsc.client_refs {
          match client_ref.as_str() {
            Some("default") => {
              esm_source.push_str(&format!(
                r#"export default registerClientReference(
function() {{ throw new Error("") }},
"{}",
"default",
)
"#,
                loader_context.resource()
              ));
            }
            Some(ident) => {
              esm_source.push_str(&format!(
                r#"export const {} = registerClientReference(
function() {{ throw new Error("") }},
"{}",
"{}",
)
"#,
                ident,
                loader_context.resource(),
                ident
              ));
            }
            _ => {}
          }
        }

        loader_context.finish_with(esm_source);
        return Ok(());
      }
    }

    for diagnostic in diagnostics {
      loader_context.emit_diagnostic(Error::warning(diagnostic).into());
    }

    if let Some(collected_ts_info) = collected_ts_info {
      loader_context.parse_meta.insert(
        COLLECTED_TYPESCRIPT_INFO_PARSE_META_KEY.to_string(),
        Box::new(collected_ts_info),
      );
    }

    // When compiling target modules, SWC retrieves the source map via sourceMapUrl.
    // The sources paths in the source map are relative to the target module. We need to resolve these paths
    // to absolute paths using the resource path to avoid incorrect project path references.
    if let (Some(map), Some(resource_dir)) = (map.as_mut(), resource_path.parent()) {
      map.set_sources(
        map
          .sources()
          .iter()
          .map(|source| {
            let source_path = Path::new(source);
            if source_path.is_relative() {
              source_path
                .absolutize_with(resource_dir.as_std_path())
                .to_string_lossy()
                .into_owned()
            } else {
              source.to_string()
            }
          })
          .collect::<Vec<_>>(),
      );
    }

    loader_context.finish_with((code, map));

    Ok(())
  }
}

pub const SWC_LOADER_IDENTIFIER: &str = "builtin:swc-loader";

#[cacheable_dyn]
#[async_trait::async_trait]
impl Loader<RunnerContext> for SwcLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }

  #[tracing::instrument("loader:builtin-swc", skip_all, fields(
    perfetto.track_name = "loader:builtin-swc",
    perfetto.process_name = "Loader Analysis",
    resource =loader_context.resource(),
  ))]
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    #[allow(unused_mut)]
    let mut inner = || self.loader_impl(loader_context);
    #[cfg(all(debug_assertions, not(target_family = "wasm")))]
    {
      // Adjust stack to avoid stack overflow.
      stacker::maybe_grow(
        2 * 1024 * 1024, /* 2mb */
        4 * 1024 * 1024, /* 4mb */
        inner,
      )
    }
    #[cfg(any(not(debug_assertions), target_family = "wasm"))]
    inner()
  }
}
