use std::fmt::Debug;

use napi_derive::napi;

use rspack_core::{CompilerOptions, CompilerOptionsBuilder, DevServerOptions};

use serde::Deserialize;

mod raw_builtins;
mod raw_context;
mod raw_define;
mod raw_entry;
mod raw_mode;
mod raw_module;
mod raw_output;
mod raw_plugins;
mod raw_resolve;
mod raw_target;

pub use raw_builtins::*;
pub use raw_context::*;
pub use raw_define::*;
pub use raw_entry::*;
pub use raw_mode::*;
pub use raw_module::*;
pub use raw_output::*;
pub use raw_plugins::*;
pub use raw_resolve::*;
pub use raw_target::*;

pub trait RawOption<T> {
  fn to_compiler_option(self, options: &CompilerOptionsBuilder) -> anyhow::Result<T>;
  /// use to create default value when input is `None`.
  fn fallback_value(options: &CompilerOptionsBuilder) -> Self;
  fn raw_to_compiler_option(
    raw: Option<Self>,
    options: &CompilerOptionsBuilder,
  ) -> anyhow::Result<T>
  where
    Self: Sized + Debug,
  {
    match raw {
      Some(value) => value,
      None => Self::fallback_value(options),
    }
    .to_compiler_option(options)
  }
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
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
  pub module: Option<RawModuleOptions>,
  pub builtins: Option<RawBuiltins>,
  #[napi(ts_type = "Record<string, string>")]
  pub define: Option<RawDefine>,
}

pub fn normalize_bundle_options(raw_options: RawOptions) -> anyhow::Result<CompilerOptions> {
  // normalize_options should ensuring orderliness.
  let compiler_options = CompilerOptionsBuilder::default()
    .then(|mut options| {
      let context = RawOption::raw_to_compiler_option(raw_options.context, &options)?;
      options.context = Some(context);
      Ok(options)
    })?
    .then(|mut options| {
      let mode = RawOption::raw_to_compiler_option(raw_options.mode, &options)?;
      options.mode = Some(mode);
      Ok(options)
    })?
    .then(|mut options| {
      let entry = RawOption::raw_to_compiler_option(raw_options.entry, &options)?;
      options.entry = Some(entry);
      Ok(options)
    })?
    .then(|mut options| {
      let output = RawOption::raw_to_compiler_option(raw_options.output, &options)?;
      options.output = Some(output);
      Ok(options)
    })?
    .then(|mut options| {
      let target = RawOption::raw_to_compiler_option(raw_options.target, &options)?;
      options.target = Some(target);
      Ok(options)
    })?
    .then(|mut options| {
      let resolve = RawOption::raw_to_compiler_option(raw_options.resolve, &options)?;
      options.resolve = Some(resolve);
      Ok(options)
    })?
    .then(|mut options| {
      let mut plugins = RawOption::raw_to_compiler_option(raw_options.plugins, &options)?;
      raw_options
        .builtins
        .as_ref()
        .map(|builtins| -> anyhow::Result<()> {
          builtins
            .html
            .as_ref()
            .map(|config| -> anyhow::Result<()> {
              let str = serde_json::to_string(&config)?;
              let configs: Vec<rspack_plugin_html::config::HtmlPluginConfig> =
                serde_json::from_str(&str)?;
              for config in configs {
                plugins.push(Box::new(rspack_plugin_html::HtmlPlugin::new(config)));
              }
              Ok(())
            })
            .transpose()?;
          Ok(())
        })
        .transpose()?;
      options.plugins = Some(plugins);
      Ok(options)
    })?
    .then(|mut options| {
      // TODO: remove or keep.
      let dev_server = DevServerOptions { hmr: false };
      options.dev_server = Some(dev_server);
      Ok(options)
    })?
    .then(|mut options| {
      let module_options = RawOption::raw_to_compiler_option(raw_options.module, &options)?;
      options.module = module_options;
      Ok(options)
    })?
    .then(|mut options| {
      let define = RawOption::raw_to_compiler_option(raw_options.define, &options)?;
      options.define = Some(define);
      Ok(options)
    })?
    .finish();

  Ok(compiler_options)
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
    assert!(&options.output.path.contains("rspack_binding_options/dist"));
  }
}
