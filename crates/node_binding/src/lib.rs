// use std::path::Path;
use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::{Env, Result};
use napi_derive::napi;
// use nodejs_resolver::Resolver;
use tokio::sync::Mutex;

// pub mod adapter;
// mod options;
mod utils;

// use adapter::utils::create_node_adapter_from_plugin_callbacks;
pub use rspack_binding_options::{normalize_bundle_options, RawOptions};

#[cfg(all(not(all(target_os = "linux", target_arch = "aarch64", target_env = "musl"))))]
#[global_allocator]
static ALLOC: mimalloc_rust::GlobalMiMalloc = mimalloc_rust::GlobalMiMalloc;

pub fn create_external<T>(value: T) -> External<T> {
  External::new(value)
}

pub type Rspack = Arc<Mutex<rspack::Compiler>>;

// #[napi(object)]
// pub struct PluginCallbacks {
//   pub build_start_callback: JsFunction,
//   pub load_callback: JsFunction,
//   pub resolve_callback: JsFunction,
//   pub build_end_callback: JsFunction,
// }

pub struct RspackBindingContext {
  pub rspack: Rspack,
  // pub resolver: Arc<Resolver>,
}

#[napi(object)]
pub struct Stats {
  // pub assets: Vec<String>,
}

impl<'a> From<rspack_core::Stats<'a>> for Stats {
  fn from(_rspack_stats: rspack_core::Stats) -> Self {
    Self {}
  }
}

#[napi]
pub fn init_trace_subscriber(env: Env) -> Result<()> {
  utils::init_custom_trace_subscriber(env)
}

#[napi(ts_return_type = "ExternalObject<RspackInternal>")]
#[allow(clippy::too_many_arguments)]
pub fn new_rspack(
  _env: Env,
  options: RawOptions,
  // plugin_callbacks: Option<PluginCallbacks>,
) -> Result<External<RspackBindingContext>> {
  // let node_adapter = create_node_adapter_from_plugin_callbacks(&env, plugin_callbacks)?;

  // let mut plugins = vec![];

  // if let Some(node_adapter) = node_adapter {
  //   plugins.push(Box::new(node_adapter) as Box<dyn rspack_core::Plugin>);
  // }

  let rspack = rspack::rspack(
    normalize_bundle_options(options).map_err(|e| Error::from_reason(format!("{:?}", e)))?,
    vec![],
  );

  // let resolver = rspack.resolver.clone();
  Ok(create_external(RspackBindingContext {
    rspack: Arc::new(Mutex::new(rspack)),
    // resolver,
  }))
}

#[napi(
  ts_args_type = "rspack: ExternalObject<RspackInternal>",
  ts_return_type = "Promise<Record<string, string>>"
)]
pub fn build(env: Env, binding_context: External<RspackBindingContext>) -> Result<napi::JsObject> {
  let compiler = binding_context.rspack.clone();
  env.execute_tokio_future(
    async move {
      let mut compiler = compiler.lock().await;
      let _rspack_stats = compiler
        .compile()
        .await
        .map_err(|e| Error::new(napi::Status::GenericFailure, format!("{:?}", e)))?;

      // let stats: Stats = rspack_stats.into();
      println!("build success");
      // Ok(stats)
      Ok(())
    },
    |_env, ret| Ok(ret),
  )
}

#[napi(
  // ts_args_type = "rspack: ExternalObject<RspackInternal>, changedFile: string[]",
  ts_args_type = "rspack: ExternalObject<RspackInternal>",
  // ts_return_type = "Promise<[diff: Record<string, string>, map: Record<string, string>]>"
  ts_return_type = "Promise<Record<string, string>>"
)]
pub fn rebuild(
  env: Env,
  binding_context: External<RspackBindingContext>,
  // changed_file: Vec<String>,
) -> Result<napi::JsObject> {
  let compiler = binding_context.rspack.clone();
  env.execute_tokio_future(
    async move {
      let mut compiler = compiler.lock().await;
      let _rspack_stats = compiler
        .compile()
        .await
        .map_err(|e| Error::new(napi::Status::GenericFailure, format!("{:?}", e)))?;

      // let stats: Stats = rspack_stats.into();
      println!("rebuild success");
      // Ok(stats)
      Ok(())
    },
    |_env, ret| Ok(ret),
  )
}

// #[napi(
//   ts_args_type = "rspack: ExternalObject<RspackInternal>, source: string, resolveOptions: ResolveOptions",
//   ts_return_type = "ResolveResult"
// )]
// pub fn resolve(
//   binding_context: External<RspackBindingContext>,
//   source: String,
//   resolve_options: ResolveOptions,
// ) -> Result<ResolveResult> {
//   let resolver = (*binding_context).resolver.clone();
//   let res = resolver.resolve(Path::new(&resolve_options.resolve_dir), &source);
//   match res {
//     Ok(val) => {
//       if let nodejs_resolver::ResolveResult::Path(p) = val {
//         Ok(ResolveResult {
//           status: true,
//           path: Some(p.to_string_lossy().to_string()),
//         })
//       } else {
//         Ok(ResolveResult {
//           status: false,
//           path: None,
//         })
//       }
//     }
//     Err(err) => Err(Error::new(Status::GenericFailure, err)),
//   }
// }

// #[napi(object)]
// pub struct ResolveOptions {
//   pub resolve_dir: String,
// }

// #[napi(object)]
// pub struct ResolveResult {
//   pub status: bool,
//   pub path: Option<String>,
// }

// #[napi]
// pub fn resolve_file(base_dir: String, import_path: String) -> Result<String> {
//   let resolver = Resolver::new(nodejs_resolver::ResolverOptions {
//     extensions: vec!["less", "css", "scss", "sass", "js"]
//       .into_iter()
//       .map(|s| s.to_owned())
//       .collect(),
//     ..Default::default()
//   });
//   match resolver.resolve(Path::new(&base_dir), &import_path) {
//     Ok(res) => {
//       if let nodejs_resolver::ResolveResult::Path(abs_path) = res {
//         match abs_path.to_str() {
//           Some(s) => Ok(s.to_owned()),
//           None => Err(Error::new(
//             napi::Status::GenericFailure,
//             "unable to create string from path".to_owned(),
//           )),
//         }
//       } else {
//         Ok(import_path)
//       }
//     }
//     Err(msg) => Err(Error::new(Status::GenericFailure, msg)),
//   }
// }

#[napi::module_init]
fn init() {
  use backtrace::Backtrace;
  use std::panic::set_hook;

  set_hook(Box::new(|panic_info| {
    let backtrace = Backtrace::new();
    println!("Panic: {:?}\nBacktrace: {:?}", panic_info, backtrace);
    std::process::exit(1)
  }));
}

// for dts generation only
#[napi(object)]
pub struct RspackInternal {}
