#[derive(Debug, Clone, Default)]
pub struct ModuleFederationManifestPluginOptions {
  pub name: Option<String>,
  pub global_name: Option<String>,
  pub file_name: Option<String>,
  pub file_path: Option<String>,
  pub stats_file_name: Option<String>,
  pub disable_assets_analyze: bool,
  pub types_file_name: Option<String>,
  pub get_public_path: Option<String>,
}
