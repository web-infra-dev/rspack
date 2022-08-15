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
mod context;
pub use context::*;
mod plugins;
pub use plugins::*;

#[derive(Debug, Default)]
pub struct CompilerOptionsBuilder {
  pub entry: Option<BundleEntries>,
  pub mode: Option<Mode>,
  pub context: Option<Context>,
  pub dev_server: Option<DevServerOptions>,
  pub output: Option<OutputOptions>,
  pub target: Option<Target>,
  pub resolve: Option<Resolve>,
  pub plugins: Option<Plugins>,
}

impl CompilerOptionsBuilder {
  pub fn unwrap(self) -> CompilerOptions {
    CompilerOptions {
      entry: self.entry.unwrap(),
      context: self.context.unwrap(),
      dev_server: self.dev_server.unwrap(),
      output: self.output.unwrap(),
      target: self.target.unwrap(),
      resolve: self.resolve.unwrap(),
      plugins: self.plugins.unwrap(),
    }
  }

  pub fn then<F: FnOnce(Self) -> Self>(self, f: F) -> Self {
    f(self)
  }
}
