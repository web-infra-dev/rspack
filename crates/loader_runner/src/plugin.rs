use rspack_error::Result;

use crate::{Content, ResourceData};

#[async_trait::async_trait]
pub trait LoaderRunnerPlugin: Send + Sync {
  async fn process_resource(&self, resource_data: &ResourceData) -> Result<Option<Content>>;
}
