use rspack_core::{CompilerOptions, Filename, OutputOptions, Plugin, PublicPath, Resolve, Target};
use rspack_plugin_html::config::HtmlPluginConfig;
use schemars::JsonSchema;
use serde::Deserialize;
use std::{collections::HashMap, path::Path, str::FromStr};

#[derive(Deserialize, Debug, Default, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestOptions {
  pub entry: HashMap<String, String>,
  pub context: Option<String>,
  pub plugins: Option<Vec<serde_json::Value>>,
}

impl From<TestOptions> for CompilerOptions {
  fn from(op: TestOptions) -> Self {
    let target = Target::from_str("web").unwrap();
    let resolve = Resolve::default();

    let output = {
      let filename = format!("{}{}{}", "[name]", "", "[ext]");

      let chunk_filename = filename.replace("[name]", "[id]");
      let path = Path::new(&op.context.clone().unwrap())
        .join("dist")
        .to_string_lossy()
        .to_string();
      let unique_name = String::from("__rspack_runtime__");
      let public_path = String::from("/");
      let asset_module_filename = format!("assets/{}", filename);
      OutputOptions {
        path,
        asset_module_filename: Filename::from_str(&asset_module_filename).unwrap(),
        filename: Filename::from_str(&filename).unwrap(),
        chunk_filename: Filename::from_str(&chunk_filename).unwrap(),
        unique_name,
        public_path: PublicPath::from_str(&public_path).unwrap(),
      }
    };

    CompilerOptions {
      entry: op
        .entry
        .into_iter()
        .map(|(name, value)| (name, value.into()))
        .collect(),
      context: op.context.unwrap(),
      plugins: create_plugins(op.plugins.unwrap_or_default()),
      resolve,
      target,
      dev_server: Default::default(),
      output,
      module: Default::default(),
    }
  }
}

impl TestOptions {
  pub fn from_fixture(fixture_path: &Path) -> Self {
    let pkg_path = fixture_path.join("test.config.json");
    let mut options = if pkg_path.exists() {
      let pkg_content = std::fs::read_to_string(pkg_path).unwrap();
      let options: TestOptions = serde_json::from_str(&pkg_content).unwrap();
      options
    } else {
      TestOptions {
        entry: HashMap::from([(
          "main".to_string(),
          fixture_path.join("index.js").to_str().unwrap().to_string(),
        )]),
        ..Default::default()
      }
    };
    assert!(
      options.context.is_none(),
      "You should not specify `root` in config. It would probably resolve to a wrong path"
    );
    options.context = Some(fixture_path.to_str().unwrap().to_string());
    options
  }
}

pub fn create_plugins(plugins: Vec<serde_json::Value>) -> Vec<Box<dyn Plugin>> {
  plugins
    .into_iter()
    .enumerate()
    .map(|(index, value)| {
      let (name, options) = if let serde_json::Value::String(name) = value {
        (name, None)
      } else {
        let (name, options): (String, serde_json::Value) = serde_json::from_value(value).unwrap();
        (name, Some(options))
      };

      (match name.as_str() {
        "html" => {
          let config: HtmlPluginConfig = match options {
            Some(config) => serde_json::from_value::<HtmlPluginConfig>(config).unwrap(),
            None => Default::default(),
          };
          Box::new(rspack_plugin_html::HtmlPlugin::new(config))
        }
        _ => {
          panic!("`config.plugins[{index}]`: plugin:{name} is not found.")
        }
      }) as Box<dyn Plugin>
    })
    .collect()
}
