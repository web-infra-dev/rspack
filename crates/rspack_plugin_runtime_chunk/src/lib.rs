#![feature(let_chains)]

use std::fmt;

use futures::future::BoxFuture;
use rspack_core::{ApplyContext, Compilation, CompilationAddEntry, CompilerOptions, PluginContext};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

#[plugin]
#[derive(Debug)]
pub struct RuntimeChunkPlugin {
  name: RuntimeChunkName,
}

impl RuntimeChunkPlugin {
  pub fn new(options: RuntimeChunkOptions) -> Self {
    Self::new_inner(options.name)
  }
}

#[derive(Debug)]
pub struct RuntimeChunkOptions {
  pub name: RuntimeChunkName,
}

pub enum RuntimeChunkName {
  Single,
  Multiple,
  String(String),
  Fn(RuntimeChunkNameFn),
}

impl fmt::Debug for RuntimeChunkName {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Single => write!(f, "Single"),
      Self::Multiple => write!(f, "Multiple"),
      Self::String(arg0) => f.debug_tuple("String").field(arg0).finish(),
      Self::Fn(_) => f.debug_tuple("Fn").finish(),
    }
  }
}

pub type RuntimeChunkNameFn =
  Box<dyn for<'a> Fn(&'a str) -> BoxFuture<'a, Result<String>> + Sync + Send>;

#[plugin_hook(CompilationAddEntry for RuntimeChunkPlugin)]
async fn add_entry(&self, compilation: &mut Compilation, entry_name: Option<&str>) -> Result<()> {
  if let Some(entry_name) = entry_name
    && let Some(data) = compilation.entries.get_mut(entry_name)
    && data.options.runtime.is_none()
    && data.options.depend_on.is_none()
  {
    let name = match &self.name {
      RuntimeChunkName::Single => "runtime".to_string(),
      RuntimeChunkName::Multiple => {
        format!("runtime~{entry_name}")
      }
      RuntimeChunkName::String(name) => name.clone(),
      RuntimeChunkName::Fn(f) => f(entry_name).await?,
    };
    data.options.runtime = Some(name.into());
  }
  Ok(())
}

impl rspack_core::Plugin for RuntimeChunkPlugin {
  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .add_entry
      .tap(add_entry::new(self));
    Ok(())
  }
}
