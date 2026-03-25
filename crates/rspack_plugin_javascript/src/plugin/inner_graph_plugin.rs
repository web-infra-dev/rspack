use rspack_core::{ApplyContext, Plugin};
use rspack_error::Result;
use rspack_hook::plugin;

#[plugin]
#[derive(Debug, Default)]
pub struct InnerGraphPlugin;

impl Plugin for InnerGraphPlugin {
  fn name(&self) -> &'static str {
    "InnerGraphPlugin"
  }

  fn apply(&self, _ctx: &mut ApplyContext<'_>) -> Result<()> {
    Ok(())
  }
}
