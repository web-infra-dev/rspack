use crate::{
  BundleEntries, Context, DevServerOptions, ModuleOptions, OutputOptions, Plugins, Resolve, Target,
};

#[derive(Debug)]
pub struct CompilerOptions {
  pub entry: BundleEntries,
  pub context: Context,
  pub dev_server: DevServerOptions,
  pub output: OutputOptions,
  pub target: Target,
  pub resolve: Resolve,
  pub plugins: Plugins,
  pub module: ModuleOptions,
}
