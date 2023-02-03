use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  str::FromStr,
};

use rspack_core::CompilerOptions;
use rspack_regex::RspackRegex;
use schemars::JsonSchema;
use serde::Deserialize;

macro_rules! impl_serde_default {
  ($name:ident) => {
    impl Default for $name {
      fn default() -> Self {
        serde_json::from_str("{}").expect("Failed to parse default config")
      }
    }
  };
}

fn default_entry() -> HashMap<String, EntryItem> {
  let mut map = HashMap::new();
  map.insert(
    "main".to_string(),
    EntryItem {
      import: vec!["./index.js".to_string()],
      runtime: None,
    },
  );
  map
}

fn default_target() -> Vec<String> {
  vec!["web".to_string(), "es2022".to_string()]
}
fn enable_runtime_by_default() -> Option<String> {
  Some("runtime".to_string())
}

fn default_chunk_filename() -> String {
  "[name][ext]".to_string()
}

fn true_by_default() -> bool {
  true
}

/// The configuration is used to configure the test in Rust.
/// The structure should be closed to the webpack configuration.
#[derive(Debug, JsonSchema, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestConfig {
  #[serde(default = "default_entry")]
  pub entry: HashMap<String, EntryItem>,
  #[serde(default)]
  pub builtins: Builtins,
  #[serde(default = "default_target")]
  pub target: Vec<String>,
  #[serde(default)]
  pub debug: Debug,
  #[serde(default)]
  pub output: Output,
  #[serde(default)]
  pub module: Module,
  #[serde(default)]
  pub optimization: Optimization,
}

#[derive(Debug, JsonSchema, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Optimization {
  // True by default to reduce code in snapshots.
  #[serde(default = "true_by_default")]
  pub remove_available_modules: bool,
}

#[derive(Debug, JsonSchema, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EntryItem {
  pub import: Vec<String>,
  #[serde(default = "enable_runtime_by_default")]
  pub runtime: Option<String>,
}

#[derive(Debug, JsonSchema, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Builtins {
  #[serde(default)]
  pub define: HashMap<String, String>,
}

#[derive(Debug, JsonSchema, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Debug {
  #[serde(default)]
  pub treeshake: bool,
  #[serde(default)]
  pub side_effects: bool,
}

#[derive(Debug, JsonSchema, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Output {
  #[serde(default = "default_chunk_filename")]
  pub filename: String,
  #[serde(default = "default_chunk_filename")]
  pub chunk_filename: String,
  #[serde(default = "default_chunk_filename")]
  pub css_filename: String,
  #[serde(default = "default_chunk_filename")]
  pub css_chunk_filename: String,
}

#[derive(Debug, JsonSchema, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Module {
  #[serde(default)]
  pub rules: Vec<Rule>,
}

#[derive(Debug, JsonSchema, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Rule {
  pub test: Option<ModuleRule>,
  pub side_effect: Option<bool>,
}

#[derive(Debug, JsonSchema, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum ModuleRule {
  Regexp { matcher: String },
}

impl_serde_default!(TestConfig);
impl_serde_default!(Output);
impl_serde_default!(Debug);
impl_serde_default!(Builtins);
impl_serde_default!(EntryItem);
impl_serde_default!(Module);
impl_serde_default!(Optimization);

impl TestConfig {
  pub fn to_compiler_options(&self, context: PathBuf) -> CompilerOptions {
    assert!(context.is_absolute());
    let config = self.clone();
    use rspack_core as c;
    CompilerOptions {
      context: c::Context::new(context.clone()),
      entry: config
        .entry
        .into_iter()
        .map(|(k, v)| {
          (
            k,
            c::EntryItem {
              import: v.import,
              runtime: v
                .runtime
                .map(Some)
                // Splitting runtime code into a separate chunk
                // is friendly to snapshot testing.
                .unwrap_or_else(|| Some("runtime".to_string())),
            },
          )
        })
        .collect(),
      output: c::OutputOptions {
        filename: c::Filename::from_str(&config.output.filename).expect("Should exist"),
        chunk_filename: c::Filename::from_str(&config.output.chunk_filename).expect("Should exist"),
        css_filename: c::Filename::from_str(&config.output.css_filename).expect("Should exist"),
        css_chunk_filename: c::Filename::from_str(&config.output.css_chunk_filename)
          .expect("Should exist"),
        asset_module_filename: c::Filename::from_str("[hash][ext][query]").expect("Should exist"),
        public_path: c::PublicPath::String("/".to_string()),
        unique_name: "__rspack_runtime__".to_string(),
        path: context.join("dist"),
        library: None,
        strict_module_error_handling: None,
      },
      target: c::Target::new(&config.target).expect("Can't construct target"),
      resolve: Default::default(),
      builtins: c::Builtins {
        define: config.builtins.define,
        tree_shaking: config.debug.treeshake,
        side_effects: config.debug.side_effects,
        ..Default::default()
      },
      plugins: vec![Box::new(rspack_plugin_css::CssPlugin::new(
        rspack_plugin_css::plugin::CssConfig {
          preset_env: Default::default(),
          postcss: rspack_plugin_css::plugin::PostcssConfig { pxtorem: None },
          modules: rspack_plugin_css::plugin::ModulesConfig {
            locals_convention: Default::default(),
            local_ident_name: rspack_plugin_css::plugin::LocalIdentName::with_mode(None),
            exports_only: Default::default(),
          },
        },
      ))],
      module: c::ModuleOptions {
        rules: config
          .module
          .rules
          .into_iter()
          .map(|rule| c::ModuleRule {
            test: rule.test.map(|test| match test {
              ModuleRule::Regexp { matcher } => c::ModuleRuleCondition::Regexp(
                RspackRegex::new(&matcher).expect("should be valid regex"),
              ),
            }),
            side_effects: rule.side_effect,
            ..Default::default()
          })
          .collect(),
        ..Default::default()
      },
      devtool: Default::default(),
      external: Default::default(),
      external_type: c::ExternalType::Auto,
      stats: Default::default(),
      snapshot: Default::default(),
      cache: c::CacheOptions::Disabled,
      experiments: Default::default(),
      dev_server: Default::default(),
      node: c::NodeOption {
        dirname: "mock".to_string(),
      },
      __emit_error: false,
      module_ids: c::ModuleIds::Named,
      optimizations: c::Optimizations {
        remove_available_modules: config.optimization.remove_available_modules,
      },
    }
  }

  pub fn from_fixture(fixture_path: &Path) -> Self {
    let config_path = fixture_path.join("test.config.json");
    // let config = if config_path
    let test_config: TestConfig = if config_path.exists() {
      let config_content =
        std::fs::read_to_string(config_path).expect("test.config.json should exist");
      serde_json::from_str(&config_content).expect("should be valid json")
    } else {
      serde_json::from_str("{}").expect("should be valid json")
    };
    test_config
  }

  pub fn compiler_options_from_fixture(fixture_path: &Path) -> CompilerOptions {
    let test_config = Self::from_fixture(fixture_path);
    test_config.to_compiler_options(fixture_path.to_path_buf())
  }
}
