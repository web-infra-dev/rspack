use async_trait::async_trait;
use core::fmt::Debug;
use rspack_core::{BundleContext, Plugin, PluginLoadHookOutput};
pub static PLUGIN_NAME: &'static str = "rspack_progress";
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};

#[derive(Debug)]
pub struct ProgressPlugin {
  progress: ProgressBar,
}
impl ProgressPlugin {
  pub fn new() -> ProgressPlugin {
    let progress = ProgressBar::new(1024);
    ProgressPlugin { progress }
  }
}

#[async_trait]
impl Plugin for ProgressPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  async fn load(&self, _ctx: &BundleContext, id: &str) -> PluginLoadHookOutput {
    self.progress.inc(1);
    None
  }
  async fn build_end(&self, _ctx: &BundleContext) {
    self.progress.finish_and_clear();
  }
}
