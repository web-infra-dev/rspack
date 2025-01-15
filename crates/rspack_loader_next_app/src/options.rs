use rspack_cacheable::cacheable;
use serde::Deserialize;

#[cacheable]
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Options {
  pub name: String,
  pub page_path: String,
  pub app_dir: String,
  pub app_paths: Option<Vec<String>>,
  pub preferred_region: Option<String>,
  pub page_extensions: Vec<String>,
  pub base_path: String,
  pub next_config_output_path: Option<String>,
  // nextConfigExperimentalUseEarlyImport?: true
  pub middleware_config: String,
  pub project_root: String,
}
