use napi::{
  bindgen_prelude::{FromNapiValue, TypeName, ValidateNapiValue},
  Either,
};
use napi_derive::napi;
use rspack_core::{
  incremental::IncrementalPasses, CacheOptions, CompilerOptions, Context, Experiments,
  ModuleOptions, OutputOptions, References,
};

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

pub use crate::raw_resolve::*;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawOptions {
  #[napi(ts_type = "undefined | 'production' | 'development' | 'none'")]
  pub mode: Option<RawMode>,
  pub context: String,
  pub output: RawOutputOptions,
  pub resolve: RawResolveOptions,
  pub resolve_loader: RawResolveOptions,
  pub module: RawModuleOptions,
  pub optimization: RawOptimizationOptions,
  pub stats: RawStatsOptions,
  pub cache: RawCacheOptions,
  pub experiments: RawExperiments,
  pub node: Option<RawNodeOption>,
  pub profile: bool,
  pub amd: Option<String>,
  pub bail: bool,
  #[napi(js_name = "__references", ts_type = "Record<string, any>")]
  pub __references: References,
}

impl TryFrom<RawOptions> for CompilerOptions {
  type Error = rspack_error::Error;

  fn try_from(value: RawOptions) -> Result<Self, rspack_error::Error> {
    let context: Context = value.context.into();
    let output: OutputOptions = value.output.try_into()?;
    let resolve = value.resolve.try_into()?;
    let resolve_loader = value.resolve_loader.try_into()?;
    let mode = value.mode.unwrap_or_default().into();
    let module: ModuleOptions = value.module.try_into()?;
    let cache = value.cache.into();
    let mut experiments: Experiments = value.experiments.into();
    if let CacheOptions::Disabled = cache {
      experiments.incremental = IncrementalPasses::empty();
    }
    let optimization = value.optimization.try_into()?;
    let stats = value.stats.into();
    let node = value.node.map(|n| n.into());

    Ok(CompilerOptions {
      context,
      mode,
      module,
      output,
      resolve,
      resolve_loader,
      experiments,
      stats,
      cache,
      optimization,
      node,
      profile: value.profile,
      amd: value.amd,
      bail: value.bail,
      __references: value.__references,
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
    Either::from_napi_value(env, napi_val).map(|either| match either {
      Either::A(false) => WithFalse::False,
      Either::A(true) => panic!("true is not a valid value for `WithFalse`"),
      Either::B(value) => WithFalse::True(value),
    })
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
    Either::from_napi_value(env, napi_val).map(|either| match either {
      Either::A(false) => WithBool::False,
      Either::A(true) => WithBool::True,
      Either::B(value) => WithBool::Value(value),
    })
  }
}
