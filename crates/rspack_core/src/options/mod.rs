mod compiler_options;
pub use compiler_options::*;
mod entry;
pub use entry::*;
mod optimization;
pub use optimization::*;
mod dev_server;
pub use dev_server::*;
mod output;
pub use output::*;
mod target;
pub use target::*;
mod resolve;
pub use resolve::*;
mod mode;
pub use mode::*;
mod builtins;
pub use builtins::*;
mod context;
pub use context::*;
mod plugins;
pub use plugins::*;
mod module;
pub use module::*;
mod define;
pub use define::*;
mod external;
pub use external::*;

#[derive(Debug, Default)]
pub struct CompilerOptionsBuilder {
  pub entry: Option<BundleEntries>,
  pub mode: Option<Mode>,
  pub context: Option<Context>,
  pub dev_server: Option<DevServerOptions>,
  pub output: Option<OutputOptions>,
  pub target: Option<Target>,
  pub resolve: Option<Resolve>,
  pub builtins: Option<Builtins>,
  pub plugins: Option<Plugins>,
  pub module: Option<ModuleOptions>,
  pub define: Option<Define>,
  pub external: Option<Vec<External>>,
}

impl CompilerOptionsBuilder {
  /// ## Warning
  /// Caller should ensure that all fields of [CompilerOptionsBuilder] are not `None`.
  /// Otherwise, this function will panic during the runtime
  pub fn finish(self) -> CompilerOptions {
    CompilerOptions {
      entry: self.entry.unwrap(),
      context: self.context.unwrap(),
      dev_server: self.dev_server.unwrap(),
      output: self.output.unwrap(),
      target: self.target.unwrap(),
      resolve: self.resolve.unwrap(),
      builtins: self.builtins.unwrap(),
      plugins: self.plugins.unwrap(),
      module: self.module.unwrap(),
      define: self.define.unwrap(),
      external: self.external.unwrap(),
      emit_error: false,
    }
  }

  pub fn then<F: FnOnce(Self) -> anyhow::Result<Self>>(self, f: F) -> anyhow::Result<Self> {
    f(self)
  }
}
