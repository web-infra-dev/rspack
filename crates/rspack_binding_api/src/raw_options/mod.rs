use std::sync::Arc;

use napi::{
  Either,
  bindgen_prelude::{FromNapiValue, TypeName, ValidateNapiValue},
};
use napi_derive::napi;
use rspack_core::{
  CacheOptions, CompilerOptions, Context, Experiments, ModuleOptions, NodeDirnameOption,
  NodeFilenameOption, NodeGlobalOption, NodeOption, OutputOptions, References,
  incremental::{IncrementalOptions, IncrementalPasses},
};
use rspack_error::error;
use rustc_hash::FxHashMap as HashMap;

mod raw_builtins;
mod raw_cache;
mod raw_devtool;
mod raw_dynamic_entry;
mod raw_experiments;
mod raw_external;
mod raw_mode;
mod raw_module;
mod raw_node;
mod raw_optimization;
mod raw_output;
mod raw_split_chunks;
mod raw_stats;

pub use raw_builtins::*;
pub use raw_cache::*;
pub use raw_devtool::*;
pub use raw_dynamic_entry::*;
pub use raw_experiments::*;
pub use raw_external::*;
pub use raw_mode::*;
pub use raw_module::*;
pub use raw_node::*;
pub use raw_optimization::*;
pub use raw_output::*;
pub use raw_split_chunks::*;
pub use raw_stats::*;

pub use crate::options::raw_resolve::*;
use crate::virtual_modules::JsVirtualFile;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawOptions {
  pub name: Option<String>,
  #[napi(ts_type = "undefined | 'production' | 'development' | 'none'")]
  pub mode: Option<RawMode>,
  pub context: String,
  pub output: RawOutputOptions,
  pub resolve: RawResolveOptions,
  pub resolve_loader: RawResolveOptions,
  pub module: RawModuleOptions,
  pub optimization: RawOptimizationOptions,
  pub stats: RawStatsOptions,
  // For now, memory.max_generation will not be exposed to the js side.
  #[napi(
    ts_type = r#"boolean | { type: "memory" } | ({ type: "persistent" } & RawCacheOptionsPersistent)"#
  )]
  pub cache: RawCacheOptions,
  pub experiments: RawExperiments,
  #[napi(ts_type = "false | { [key: string]: boolean }")]
  pub incremental: Option<WithFalse<RawIncremental>>,
  pub node: Option<RawNodeOption>,
  pub amd: Option<String>,
  pub bail: bool,
  #[napi(js_name = "__references", ts_type = "Record<string, string>")]
  pub __references: HashMap<String, String>,
  #[napi(js_name = "__virtual_files")]
  pub __virtual_files: Option<Vec<JsVirtualFile>>,
}

fn normalize_raw_node_option(
  node: Option<RawNodeOption>,
) -> rspack_error::Result<Option<NodeOption>> {
  node
    .map(|n| {
      let dirname = match n.dirname.as_str() {
        "mock" => NodeDirnameOption::Mock,
        "warn-mock" => NodeDirnameOption::WarnMock,
        "eval-only" => NodeDirnameOption::EvalOnly,
        "node-module" => NodeDirnameOption::NodeModule,
        "true" => NodeDirnameOption::True,
        "false" => NodeDirnameOption::False,
        _ => return Err(error!("invalid node.dirname: {}", n.dirname.as_str())),
      };
      let filename = match n.filename.as_str() {
        "mock" => NodeFilenameOption::Mock,
        "warn-mock" => NodeFilenameOption::WarnMock,
        "eval-only" => NodeFilenameOption::EvalOnly,
        "node-module" => NodeFilenameOption::NodeModule,
        "true" => NodeFilenameOption::True,
        "false" => NodeFilenameOption::False,
        _ => return Err(error!("invalid node.filename: {}", n.filename.as_str())),
      };
      let global = match n.global.as_str() {
        "true" => NodeGlobalOption::True,
        "warn" => NodeGlobalOption::Warn,
        "false" => NodeGlobalOption::False,
        _ => return Err(error!("invalid node.global: {}", n.global.as_str())),
      };
      Ok(NodeOption {
        dirname,
        filename,
        global,
      })
    })
    .transpose()
}

impl TryFrom<RawOptions> for CompilerOptions {
  type Error = rspack_error::Error;

  fn try_from(value: RawOptions) -> Result<Self, rspack_error::Error> {
    let RawOptions {
      name,
      mode,
      context,
      output,
      resolve,
      resolve_loader,
      module,
      optimization,
      stats,
      cache,
      experiments,
      incremental,
      node,
      amd,
      bail,
      __references,
      __virtual_files: _,
    } = value;

    let context: Context = context.into();
    let mode = mode.unwrap_or_default().into();
    let cache = normalize_raw_cache(cache);
    let experiments: Experiments = experiments.into();
    let mut incremental: IncrementalOptions = match incremental {
      Some(value) => match value {
        WithFalse::True(value) => value.into(),
        WithFalse::False => IncrementalOptions::empty_passes(),
      },
      None => IncrementalOptions::empty_passes(),
    };
    if let CacheOptions::Disabled = cache {
      incremental.passes = IncrementalPasses::empty();
    }
    let stats = stats.into();
    let node = normalize_raw_node_option(node)?;
    let __references: References = __references
      .into_iter()
      .map(|(key, value)| (key, Arc::<str>::from(value)))
      .collect();

    let converted = rayon::join(
      || {
        rayon::join(
          || output.try_into(),
          || rayon::join(|| resolve.try_into(), || resolve_loader.try_into()),
        )
      },
      || rayon::join(|| module.try_into(), || optimization.try_into()),
    );

    let (output, resolve, resolve_loader, module, optimization): (
      OutputOptions,
      _,
      _,
      ModuleOptions,
      _,
    ) = {
      let ((output, (resolve, resolve_loader)), (module, optimization)) = converted;
      (output?, resolve?, resolve_loader?, module?, optimization?)
    };

    Ok(CompilerOptions {
      name,
      context,
      mode,
      module,
      output,
      resolve,
      resolve_loader,
      experiments,
      incremental,
      stats,
      cache,
      optimization,
      node,
      amd,
      bail,
      __references,
    })
  }
}

#[derive(Debug)]
pub enum WithFalse<T> {
  False,
  True(T),
}

impl<T: ValidateNapiValue + FromNapiValue> FromNapiValue for WithFalse<T> {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    unsafe {
      Either::from_napi_value(env, napi_val).map(|either| match either {
        Either::A(false) => WithFalse::False,
        Either::A(true) => panic!("true is not a valid value for `WithFalse`"),
        Either::B(value) => WithFalse::True(value),
      })
    }
  }
}

impl<T: ValidateNapiValue> ValidateNapiValue for WithFalse<T> {}

impl<T: TypeName> TypeName for WithFalse<T> {
  fn type_name() -> &'static str {
    T::type_name()
  }

  fn value_type() -> napi::ValueType {
    T::value_type()
  }
}

#[derive(Default, Debug)]
pub enum WithBool<T> {
  True,
  #[default]
  False,
  Value(T),
}

impl<T> WithBool<T> {
  pub fn as_bool(&self) -> Option<bool> {
    match self {
      WithBool::True => Some(true),
      WithBool::False => Some(false),
      WithBool::Value(_) => None,
    }
  }

  pub fn as_value(&self) -> Option<&T> {
    match self {
      WithBool::Value(value) => Some(value),
      _ => None,
    }
  }
}

impl<T: ValidateNapiValue + FromNapiValue> FromNapiValue for WithBool<T> {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    unsafe {
      Either::from_napi_value(env, napi_val).map(|either| match either {
        Either::A(false) => WithBool::False,
        Either::A(true) => WithBool::True,
        Either::B(value) => WithBool::Value(value),
      })
    }
  }
}
