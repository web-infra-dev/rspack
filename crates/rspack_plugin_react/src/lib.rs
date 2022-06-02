#![deny(clippy::all)]

mod react_hmr;
use async_trait::async_trait;
use react_hmr::{
  load_hmr_runtime_path, FoundReactRefreshVisitor, InjectReactRefreshEntryFloder, ReactHmrFolder,
  HMR_ENTRY, HMR_ENTRY_PATH, HMR_RUNTIME_PATH,
};
use rspack_core::ast::Ident;
use rspack_core::path::normalize_path;
use rspack_core::{
  ast, BundleContext, BundleMode, LoadArgs, LoadedSource, Loader, OnResolveResult, Plugin,
  PluginLoadHookOutput, PluginResolveHookOutput, ResolveArgs,
};
use rspack_swc::swc_common::comments::SingleThreadedComments;
use rspack_swc::swc_ecma_transforms_base::resolver;
use rspack_swc::swc_ecma_transforms_react::{react, Options, RefreshOptions, Runtime};
use rspack_swc::swc_ecma_visit::{
  noop_visit_mut_type, FoldWith, VisitMut, VisitMutWith, VisitWith,
};
use std::path::Path;
use swc_common::{Mark, SyntaxContext};

pub static PLUGIN_NAME: &str = "rspack_plugin_react";

#[derive(Debug)]
pub struct ReactPlugin {
  pub runtime: Runtime,
}

#[async_trait]
impl Plugin for ReactPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  #[inline]
  fn need_build_start(&self) -> bool {
    false
  }

  #[inline]
  fn need_build_end(&self) -> bool {
    false
  }

  #[inline]
  fn need_transform(&self) -> bool {
    false
  }

  #[inline]
  fn need_tap_generated_chunk(&self) -> bool {
    false
  }
  async fn resolve(&self, _ctx: &BundleContext, args: &ResolveArgs) -> PluginResolveHookOutput {
    if args.id == HMR_RUNTIME_PATH || args.id == HMR_ENTRY_PATH {
      Ok(Some(OnResolveResult {
        uri: args.id.to_string(),
        external: false,
      }))
    } else {
      Ok(None)
    }
  }

  async fn load(&self, _ctx: &BundleContext, args: &LoadArgs) -> PluginLoadHookOutput {
    if args.id == HMR_RUNTIME_PATH {
      return Ok(Some(LoadedSource::with_loader(
        load_hmr_runtime_path(&_ctx.options.as_ref().root),
        Loader::Js,
      )));
    } else if args.id == HMR_ENTRY_PATH {
      return Ok(Some(LoadedSource::with_loader(
        HMR_ENTRY.to_string(),
        Loader::Js,
      )));
    } else {
      return Ok(None);
    }
  }

  fn transform_ast(
    &self,
    ctx: &rspack_core::PluginContext,
    uri: &str,
    mut ast: ast::Module,
  ) -> rspack_core::PluginTransformAstHookOutput {
    let id = uri;

    if ctx.options.react.refresh {
      let is_entry = ctx.is_entry_uri(uri);
      println!(
        "path: {:?}, ctx.options.entries: {:?}, is_entry {:?}",
        uri, ctx.options.entries, is_entry
      );
      if is_entry {
        ast = ast.fold_with(&mut InjectReactRefreshEntryFloder {});
      }
    }

    let is_node_module = id.contains("node_modules");
    let is_maybe_has_jsx = Path::new(uri).extension().map_or(true, |ext| ext != "ts");
    if is_maybe_has_jsx {
      ctx.compiler.run(|| {
        let (unresolved_mark, top_level_mark) = (Mark::new(), Mark::new());
        if !is_node_module {
          // The Resolver is not send. We need this block to tell compiler that
          // the Resolver won't be sent over the threads
          ast.visit_mut_with(&mut ClearMark);
          ast.visit_mut_with(&mut resolver(unresolved_mark, top_level_mark, false));
        }
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
          top_level_mark,
        );

        ast = ast.fold_with(&mut react_folder);

        if !ctx.options.react.refresh {
          return Ok(ast);
        }

        let result = match is_node_module {
          true => ast,
          false => {
            let mut f = FoundReactRefreshVisitor {
              is_refresh_boundary: false,
            };
            ast.visit_with(&mut f);
            match f.is_refresh_boundary {
              true => ast.fold_with(&mut ReactHmrFolder {
                id: normalize_path(id, ctx.options.as_ref().root.as_str()),
              }),
              false => ast,
            }
          }
        };

        Ok(result)
      })
    } else {
      Ok(ast)
    }
  }
}

#[derive(Clone, Copy)]
struct ClearMark;
impl VisitMut for ClearMark {
  noop_visit_mut_type!();

  fn visit_mut_ident(&mut self, ident: &mut Ident) {
    ident.span.ctxt = SyntaxContext::empty();
  }
}
