use crate::OutputAssetModuleFilename;

#[derive(Debug, Default)]
pub struct OutputOptions {
  pub path: String,
  pub public_path: String,
  pub asset_module_filename: OutputAssetModuleFilename,
  pub namespace: String,
}
