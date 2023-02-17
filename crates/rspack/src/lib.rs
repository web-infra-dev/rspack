#![recursion_limit = "256"]
pub use rspack_core::Compiler;
use rspack_core::{CompilerOptions, Plugin, TargetPlatform};
use rspack_error::Result;
use rspack_plugin_asset::AssetConfig;
use rspack_plugin_devtool::DevtoolPluginOptions;

pub fn rspack(mut options: CompilerOptions, mut plugins: Vec<Box<dyn Plugin>>) -> Compiler {
  // Css plugin is registered via option normalization
  plugins.push(Box::new(rspack_plugin_asset::AssetPlugin::new(
    AssetConfig {
      parse_options: options.module.parser.as_ref().and_then(|x| x.asset.clone()),
    },
  )));
  plugins.push(Box::new(rspack_plugin_json::JsonPlugin {}));
  match &options.target.platform {
    TargetPlatform::Web => {
      plugins.push(Box::new(
        rspack_plugin_runtime::ArrayPushCallbackChunkFormatPlugin {},
      ));
      plugins.push(Box::new(rspack_plugin_runtime::RuntimePlugin {}));
      plugins.push(Box::new(rspack_plugin_runtime::CssModulesPlugin {}));
      plugins.push(Box::new(rspack_plugin_runtime::JsonpChunkLoadingPlugin {}));
    }
    TargetPlatform::Node(_) => {
      plugins.push(Box::new(
        rspack_plugin_runtime::CommonJsChunkFormatPlugin {},
      ));
      plugins.push(Box::new(rspack_plugin_runtime::RuntimePlugin {}));
      plugins.push(Box::new(
        rspack_plugin_runtime::CommonJsChunkLoadingPlugin {},
      ));
    }
    _ => {
      plugins.push(Box::new(rspack_plugin_runtime::RuntimePlugin {}));
    }
  };
  if options.dev_server.hot {
    // https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/HotModuleReplacementPlugin.js#L87-L89
    if options.output.strict_module_error_handling.is_none() {
      options.output.strict_module_error_handling = Some(true);
    }
    plugins.push(Box::new(
      rspack_plugin_runtime::HotModuleReplacementPlugin {},
    ));
  }
  plugins.push(Box::new(
    rspack_plugin_runtime::BasicRuntimeRequirementPlugin {},
  ));
  if options.experiments.lazy_compilation {
    plugins.push(Box::new(rspack_plugin_runtime::LazyCompilationPlugin {}));
  }
  plugins.push(Box::<rspack_plugin_externals::ExternalPlugin>::default());
  plugins.append(&mut options.plugins);
  plugins.push(Box::new(rspack_plugin_javascript::JsPlugin::new()));
  plugins.push(Box::new(rspack_plugin_devtool::DevtoolPlugin::new(
    DevtoolPluginOptions {
      inline: options.devtool.inline(),
      append: !options.devtool.hidden(),
      namespace: options.output.unique_name.clone(),
      columns: !options.devtool.cheap(),
      no_sources: options.devtool.no_sources(),
      public_path: None,
    },
  )));
  match options.module_ids {
    rspack_core::ModuleIds::Named => {
      plugins.push(Box::new(rspack_ids::NamedModuleIdsPlugin {}));
    }
    rspack_core::ModuleIds::Deterministic => {
      plugins.push(Box::new(rspack_ids::DeterministicModuleIdsPlugin {}));
    }
  }

  plugins.push(Box::new(rspack_ids::StableNamedChunkIdsPlugin::new(
    None, None,
  )));

  // Notice the plugin need to be placed after SplitChunksPlugin
  plugins.push(Box::new(
    rspack_plugin_remove_empty_chunks::RemoveEmptyChunksPlugin,
  ));

  if let Some(copy) = &options.builtins.copy {
    plugins.push(Box::new(rspack_plugin_copy::CopyPlugin {
      patterns: copy.patterns.clone(),
    }));
  }

  Compiler::new(options, plugins)
}

pub fn dev_server(options: CompilerOptions, plugins: Vec<Box<dyn Plugin>>) -> DevServer {
  DevServer {
    compiler: rspack(options, plugins),
  }
}

pub struct DevServer {
  compiler: Compiler,
}

impl DevServer {
  pub async fn serve(&mut self) -> Result<()> {
    self.compiler.build().await?;

    warp::fs::dir(self.compiler.options.context.join("dist"));
    let filter = warp::fs::dir(self.compiler.options.context.join("dist"));

    // let routes = warp::ws().map(|ws: warp::ws::Ws| {
    //   // And then our closure will be called when it completes...
    //   ws.on_upgrade(|websocket| {
    //     // Just echo all messages back...
    //     let (tx, rx) = websocket.split();
    //     rx.forward(tx).map(|result| {
    //       if let Err(e) = result {
    //         eprintln!("websocket error: {:?}", e);
    //       }
    //     })
    //   })
    // });

    warp::serve(filter).run(([127, 0, 0, 1], 3031)).await;
    Ok(())
  }
}
