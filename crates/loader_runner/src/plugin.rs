use rspack_error::Result;

use crate::{Content, ResourceData};

#[async_trait::async_trait]
pub trait LoaderRunnerPlugin {
  async fn process_resource(&self, resource_data: ResourceData) -> Result<Content>;
}
