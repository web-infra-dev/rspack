use futures::future::BoxFuture;
use rspack_core::CompilerId;
use rspack_error::{Result, ToStringResultToRspackResultExt};

use crate::{loaders::client_entry_loader::ClientEntry, plugin_state::PLUGIN_STATE_BY_COMPILER_ID};

type CompileFn = Box<dyn Fn() -> BoxFuture<'static, Result<()>> + Sync + Send>;

pub struct ClientCompilerHandle {
  compile_fn: CompileFn,
}

impl ClientCompilerHandle {
  pub fn new(compile_fn: CompileFn) -> Self {
    Self { compile_fn }
  }

  pub async fn compile(&self) -> Result<()> {
    let compile_fn = &self.compile_fn;
    compile_fn().await?;
    Ok(())
  }
}
