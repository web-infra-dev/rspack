use rspack_cacheable::{cacheable, with::AsPreset};
use serde::Deserialize;
use serde_json::{Map, Value, json};

#[cacheable]
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct LessLoaderOptions {
  pub source_map: Option<bool>,
  pub additional_data: Option<String>,
  #[cacheable(with=AsPreset)]
  pub less_options: Map<String, Value>,
}

impl LessLoaderOptions {
  pub(crate) fn compiler_options(&self, filename: &str, source_map: bool) -> less_rs::Options {
    // Match less-loader's defaults before passing all compiler options to less-rs.
    let mut options = Map::from_iter([
      ("relativeUrls".into(), Value::Bool(true)),
      ("filename".into(), Value::String(filename.into())),
    ]);
    options.extend(self.less_options.clone());
    if source_map {
      options.insert(
        "sourceMap".into(),
        json!({
          "sourceMapBasepath": "",
          "outputSourceFiles": true,
          "disableSourcemapAnnotation": true,
        }),
      );
    }
    less_rs::Options::from_json(&Value::Object(options))
  }
}
