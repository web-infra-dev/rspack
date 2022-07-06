use crate::OutputAssetModuleFilename;

#[derive(Debug, Default)]
pub struct OutputOptions {
  pub path: String,
  pub asset_module_filename: OutputAssetModuleFilename,
}
