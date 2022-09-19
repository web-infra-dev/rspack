use std::path::Path;

pub use rspack_core::Compiler;
use rspack_core::{CompilerOptions, ExternalPlugin, Plugin};
use rspack_error::Result;
use rspack_plugin_asset::AssetConfig;

pub fn rspack(mut options: CompilerOptions, mut plugins: Vec<Box<dyn Plugin>>) -> Compiler {
  // plugins.push(Box::new(rspack_plugin_css::CssPlugin::default()));
  plugins.push(Box::new(rspack_plugin_asset::AssetPlugin::new(
    AssetConfig {
      parse_options: options.module.parser.as_ref().and_then(|x| x.asset.clone()),
    },
  )));
  plugins.push(Box::new(rspack_plugin_json::JsonPlugin {}));
  plugins.push(Box::new(rspack_plugin_runtime::RuntimePlugin {}));
  plugins.push(Box::new(ExternalPlugin {}));
  plugins.push(Box::new(rspack_plugin_devtool::DevtoolPlugin {}));
  plugins.append(&mut options.plugins);
  plugins.push(Box::new(rspack_plugin_javascript::JsPlugin::new()));
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

    warp::fs::dir(Path::new(self.compiler.options.context.as_str()).join("dist"));
    let filter = warp::fs::dir(Path::new(self.compiler.options.context.as_str()).join("dist"));

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
