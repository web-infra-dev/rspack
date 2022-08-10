use anyhow::Result;

#[async_trait::async_trait]
pub trait LoaderRunnerPlugin {
  async fn process_resource(&self) -> Result<Option<()>>;
}
