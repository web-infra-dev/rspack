use serde::Deserialize;
pub use styled_components::styled_components;

pub type StyledComponentsOptions = styled_components::Config;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RawStyledComponentsOptions {
  pub display_name: Option<bool>,
  pub ssr: Option<bool>,
  pub file_name: Option<bool>,
  pub meaningless_file_names: Option<Vec<String>>,
  pub namespace: Option<String>,
  pub top_level_import_paths: Option<Vec<String>>,
  pub transpile_template_literals: Option<bool>,
  pub minify: Option<bool>,
  pub pure: Option<bool>,
  pub css_prop: Option<bool>,
}

impl From<RawStyledComponentsOptions> for StyledComponentsOptions {
  fn from(raw_config: RawStyledComponentsOptions) -> Self {
    Self {
      display_name: raw_config.display_name.unwrap_or(true),
      ssr: raw_config.ssr.unwrap_or(true),
      file_name: raw_config.file_name.unwrap_or(true),
      meaningless_file_names: raw_config
        .meaningless_file_names
        .unwrap_or_else(|| vec!["index".to_string()]),
      namespace: raw_config.namespace.unwrap_or_default(),
      top_level_import_paths: raw_config
        .top_level_import_paths
        .unwrap_or_default()
        .into_iter()
        .map(|s| s.into())
        .collect(),
      transpile_template_literals: raw_config.transpile_template_literals.unwrap_or_default(),
      minify: raw_config.minify.unwrap_or_default(),
      pure: raw_config.pure.unwrap_or_default(),
      css_prop: raw_config.css_prop.unwrap_or(true),
    }
  }
}
