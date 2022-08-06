use std::path::Path;

pub use rspack_core::Compiler;
use rspack_core::{CompilerOptions, Plugin};

pub fn rspack(options: CompilerOptions, mut plugins: Vec<Box<dyn Plugin>>) -> Compiler {
  plugins.push(Box::new(rspack_plugin_javascript::JsPlugin {}));
  plugins.push(Box::new(rspack_plugin_css::CssPlugin::new(
    options.css.clone(),
  )));
  plugins.push(Box::new(rspack_plugin_asset::AssetPlugin {}));
  plugins.push(Box::new(rspack_plugin_json::JsonPlugin {}));
  plugins.push(Box::new(rspack_plugin_runtime::RuntimePlugin {}));
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
  pub async fn serve(&mut self) {
    self.compiler.compile().await.unwrap();

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
  }
}
