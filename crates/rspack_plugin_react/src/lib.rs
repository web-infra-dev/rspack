mod react_hmr;
use async_trait::async_trait;
use react_hmr::{
  load_hmr_runtime_path, FoundReactRefreshVisitor, InjectReactRefreshEntryFloder, ReactHmrFolder,
  HMR_ENTRY, HMR_ENTRY_PATH, HMR_RUNTIME_PATH,
};
use rspack_core::path::normalize_path;
use rspack_core::{
  ast, BundleContext, BundleMode, LoadedSource, Loader, Plugin, PluginLoadHookOutput,
  PluginResolveHookOutput, ResolvedURI,
};
use rspack_swc::swc_common::{comments::SingleThreadedComments, GLOBALS};
use rspack_swc::swc_ecma_transforms_react::{react, Options, RefreshOptions, Runtime};
use rspack_swc::swc_ecma_visit::{FoldWith, VisitWith};
use std::path::Path;

pub static PLUGIN_NAME: &'static str = "rspack_plugin_react";

#[derive(Debug)]
pub struct ReactPlugin {
  pub runtime: Runtime,
}

#[async_trait]
impl Plugin for ReactPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  async fn resolve(
    &self,
    _ctx: &BundleContext,
    id: &str,
    _importer: Option<&str>,
  ) -> PluginResolveHookOutput {
    if id == HMR_RUNTIME_PATH || id == HMR_ENTRY_PATH {
      Some(ResolvedURI {
        uri: id.to_string(),
        external: false,
      })
    } else {
      None
    }
  }

  async fn load(&self, _ctx: &BundleContext, id: &str) -> PluginLoadHookOutput {
    if id == HMR_RUNTIME_PATH {
      return Some(LoadedSource::with_loader(
        load_hmr_runtime_path(&_ctx.options.as_ref().root),
        Loader::Js,
      ));
    } else if id == HMR_ENTRY_PATH {
      return Some(LoadedSource::with_loader(HMR_ENTRY.to_string(), Loader::Js));
    } else {
      return None;
    }
  }

  fn transform_ast(
    &self,
    ctx: &rspack_core::BundleContext,
    path: &Path,
    mut ast: ast::Module,
  ) -> rspack_core::PluginTransformAstHookOutput {
    let id = path.to_str().unwrap_or("").to_string();
    if ctx.options.react.refresh {
      let is_entry = ctx
        .options
        .entries
        .iter()
        .find(|e| e.as_str() == id)
        .is_some();

      if is_entry {
        ast = ast.fold_with(&mut InjectReactRefreshEntryFloder {});
      }
    }

    let is_node_module = id.find("node_modules").is_some();
    let is_maybe_has_jsx = path.extension().map_or(true, |ext| ext != "ts");
    if is_maybe_has_jsx {
      ctx.compiler.run(|| {
        let mut react_folder = react::<SingleThreadedComments>(
          ctx.compiler.cm.clone(),
          None,
          Options {
            development: matches!(ctx.options.mode, BundleMode::Dev).into(),
            runtime: Some(self.runtime),
            refresh: if ctx.options.react.refresh && !is_node_module {
              Some(RefreshOptions {
                ..Default::default()
              })
            } else {
              None
            },
            ..Default::default()
          },
          ctx.top_level_mark.clone(),
        );

        ast = ast.fold_with(&mut react_folder);

        if !ctx.options.react.refresh {
          return ast;
        }

        match is_node_module {
          true => ast,
          false => {
            let mut f = FoundReactRefreshVisitor {
              is_refresh_boundary: false,
            };
            ast.visit_with(&mut f);
            match f.is_refresh_boundary {
              true => ast.fold_with(&mut ReactHmrFolder {
                id: normalize_path(id.as_str(), ctx.options.as_ref().root.as_str()),
              }),
              false => ast,
            }
          }
        }
      })
    } else {
      ast
    }
  }
}
