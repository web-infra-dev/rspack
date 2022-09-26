use rspack_error::Result;

use crate::{Content, ResourceData};

pub trait LoaderRunnerPlugin: Send + Sync {
  fn name(&self) -> &'static str {
    "unknown"
  }

  fn process_resource(&self, resource_data: &ResourceData) -> Result<Option<Content>>;
}
