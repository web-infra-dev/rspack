use napi_derive::napi;
use rspack_error::Result;
use rspack_plugin_lightning_css_minimizer::LightningCssMinimizerOptions;

#[derive(Debug)]
#[napi(object)]
pub struct RawLightningCssMinimizerRspackPluginOptions {
  pub error_recovery: bool,
  pub unused_symbols: Vec<String>,
  pub remove_unused_local_idents: bool,
  pub browserslist: Vec<String>,
}

impl TryFrom<RawLightningCssMinimizerRspackPluginOptions> for LightningCssMinimizerOptions {
  type Error = rspack_error::Error;

  fn try_from(value: RawLightningCssMinimizerRspackPluginOptions) -> Result<Self> {
    Ok(Self {
      error_recovery: value.error_recovery,
      unused_symbols: value.unused_symbols,
      remove_unused_local_idents: value.remove_unused_local_idents,
      browserlist: value.browserslist,
    })
  }
}
