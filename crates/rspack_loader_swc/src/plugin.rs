use std::sync::Arc;

use rspack_hook::{plugin, plugin_hook};

#[plugin]
#[derive(Debug)]
pub struct PluginCssExtract {
  pub(crate) options: Arc<CssExtractOptions>,
}
