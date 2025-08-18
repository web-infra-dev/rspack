mod context_replacement;
mod normal_module_replacement;

pub use context_replacement::{ContextReplacementPlugin, ContextReplacementPluginOptions};
pub use normal_module_replacement::{
  NormalModuleReplacementPlugin, NormalModuleReplacementPluginOptions, NormalModuleReplacer,
};
