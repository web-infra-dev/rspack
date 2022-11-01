use crate::{
  Builtins, BundleEntries, Context, DevServerOptions, Devtool, External, ExternalType,
  ModuleOptions, OutputOptions, Plugins, Resolve, Target,
};

#[derive(Debug)]
pub struct CompilerOptions {
  pub entry: BundleEntries,
  pub context: Context,
  pub dev_server: DevServerOptions,
  pub output: OutputOptions,
  pub target: Target,
  pub resolve: Resolve,
  pub builtins: Builtins,
  pub plugins: Plugins,
  pub module: ModuleOptions,
  pub devtool: Devtool,
  pub external: Vec<External>,
  pub external_type: ExternalType,
}
