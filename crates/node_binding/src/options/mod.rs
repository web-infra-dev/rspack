use napi_derive::napi;

use napi::bindgen_prelude::*;

use rspack_core::{CompilerOptions, CompilerOptionsBuilder, DevServerOptions};

use serde::Deserialize;

mod raw_context;
mod raw_entry;
mod raw_mode;
mod raw_output;
mod raw_plugins;
mod raw_resolve;
mod raw_target;

pub use raw_context::*;
pub use raw_entry::*;
pub use raw_mode::*;
pub use raw_output::*;
pub use raw_plugins::*;
pub use raw_resolve::*;
pub use raw_target::*;

pub trait RawOption<T> {
  fn to_compiler_option(self, options: &CompilerOptionsBuilder) -> T;
  /// use to create default value when input is `None`.
  fn fallback_value(options: &CompilerOptionsBuilder) -> Self;
}
fn to_compiler_option<T: RawOption<U>, U>(value: Option<T>, options: &CompilerOptionsBuilder) -> U {
  (match value {
    Some(value) => value,
    None => T::fallback_value(options),
  })
  .to_compiler_option(options)
}
#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "test")]
#[napi(object)]
pub struct RawOptions {
  pub entry: Option<RawEntry>,
  pub mode: Option<RawMode>,
  pub target: Option<RawTarget>,
  // #[napi(ts_type = "\"browser\" | \"node\"")]
  // pub platform: Option<String>,
  pub context: Option<RawContext>,
  // pub loader: Option<HashMap<String, String>>,
  // pub enhanced: Option<RawEnhancedOptions>,
  // pub optimization: Option<RawOptimizationOptions>,
  pub output: Option<RawOutputOptions>,
  pub resolve: Option<RawResolveOptions>,
  // pub chunk_filename: Option<String>,
  pub plugins: Option<RawPlugins>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg(not(feature = "test"))]
#[napi(object)]
pub struct RawOptions {
  #[napi(ts_type = "Record<string, string>")]
  pub entry: Option<RawEntry>,
  #[napi(ts_type = "string")]
  pub mode: Option<RawMode>,
  #[napi(ts_type = "string")]
  pub target: Option<RawTarget>,
  // #[napi(ts_type = "\"browser\" | \"node\"")]
  // pub platform: Option<String>,
  #[napi(ts_type = "string")]
  pub context: Option<RawContext>,
  // pub loader: Option<HashMap<String, String>>,
  // pub enhanced: Option<RawEnhancedOptions>,
  // pub optimization: Option<RawOptimizationOptions>,
  pub output: Option<RawOutputOptions>,
  pub resolve: Option<RawResolveOptions>,
  // pub chunk_filename: Option<String>,
  #[napi(ts_type = "any[]")]
  pub plugins: Option<RawPlugins>,
}

pub fn normalize_bundle_options(raw_options: RawOptions) -> Result<CompilerOptions> {
  // normalize_options should ensuring orderliness.
  let compier_options = CompilerOptionsBuilder::default()
    .then(|mut options| {
      let context = to_compiler_option(raw_options.context, &options);
      options.context = Some(context);
      options
    })
    .then(|mut options| {
      let mode = to_compiler_option(raw_options.mode, &options);
      options.mode = Some(mode);
      options
    })
    .then(|mut options| {
      let entry = to_compiler_option(raw_options.entry, &options);
      options.entry = Some(entry);
      options
    })
    .then(|mut options| {
      let output = to_compiler_option(raw_options.output, &options);
      options.output = Some(output);
      options
    })
    .then(|mut options| {
      let target = to_compiler_option(raw_options.target, &options);
      options.target = Some(target);
      options
    })
    .then(|mut options| {
      let resolve = to_compiler_option(raw_options.resolve, &options);
      options.resolve = Some(resolve);
      options
    })
    .then(|mut options| {
      let plugins = to_compiler_option(raw_options.plugins, &options).unwrap();
      options.plugins = Some(plugins);
      options
    })
    .then(|mut options| {
      // TODO: remove or keep.
      let dev_server = DevServerOptions { hmr: false };
      options.dev_server = Some(dev_server);
      options
    })
    .unwrap();

  Ok(compier_options)
}

// pub fn parse_raw_alias(
//   alias: HashMap<String, ResolveAliasValue>,
// ) -> HashMap<String, Option<String>> {
//   HashMap::from_iter(
//     alias
//       .into_iter()
//       .map(|(key, value)| {
//         let value = match value {
//           ResolveAliasValue::False(b) => {
//             if b {
//               panic!("alias should not be true");
//             } else {
//               None
//             }
//           }
//           ResolveAliasValue::Target(s) => Some(s),
//         };
//         (key, value)
//       })
//       .collect::<Vec<_>>(),
//   )
// }

// pub fn parse_raw_condition_names(condition_names: Vec<String>) -> HashSet<String> {
//   HashSet::from_iter(condition_names.into_iter())
// }

#[cfg(test)]
mod test {
  use crate::normalize_bundle_options;

  #[test]
  fn empty_test() {
    let raw = serde_json::from_str("{}").unwrap();
    let options = normalize_bundle_options(raw).unwrap();
    assert!(&options.output.path.contains("node_binding/dist"));
  }
}
