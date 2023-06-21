use std::sync::atomic::Ordering::SeqCst;
use std::sync::RwLock;
use std::{cmp, sync::atomic::AtomicU32};

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use rspack_core::{
  Compilation, DoneArgs, MakeParam, Module, OptimizeChunksArgs, Plugin, PluginBuildEndHookOutput,
  PluginContext, PluginMakeHookOutput, PluginOptimizeChunksOutput, PluginProcessAssetsOutput,
  ProcessAssetsArgs,
};
use rspack_error::Result;

#[derive(Debug, Clone, Default)]
pub struct ProgressPluginConfig {
  // the prefix name of progress bar
  pub prefix: Option<String>,
}

#[derive(Debug)]
pub struct ProgressPlugin {
  pub options: ProgressPluginConfig,
  pub progress_bar: ProgressBar,
  pub modules_count: AtomicU32,
  pub modules_done: AtomicU32,
  pub last_modules_count: RwLock<Option<u32>>,
}

impl ProgressPlugin {
  pub fn new(options: ProgressPluginConfig) -> Self {
    let progress_bar = ProgressBar::with_draw_target(Some(100), ProgressDrawTarget::stdout());
    progress_bar.set_style(
      ProgressStyle::with_template("{prefix} {bar:40.cyan/blue} {percent}% {wide_msg}")
        .expect("TODO:"),
    );
    Self {
      options,
      progress_bar,
      modules_count: AtomicU32::new(0),
      modules_done: AtomicU32::new(0),
      last_modules_count: RwLock::new(None),
    }
  }
}

#[async_trait::async_trait]
impl Plugin for ProgressPlugin {
  fn name(&self) -> &'static str {
    "progress"
  }

  async fn make(
    &self,
    _ctx: PluginContext,
    _compilation: &mut Compilation,
    _param: &mut MakeParam,
  ) -> PluginMakeHookOutput {
    self.progress_bar.reset();
    self.progress_bar.set_prefix(
      self
        .options
        .prefix
        .clone()
        .unwrap_or_else(|| "Rspack".to_string()),
    );
    self.modules_count.store(0, SeqCst);
    self.modules_done.store(0, SeqCst);
    self.progress_bar.set_message("make");
    self.progress_bar.set_position(1);
    Ok(())
  }

  async fn build_module(&self, module: &mut dyn Module) -> Result<()> {
    if let Some(module) = module.as_normal_module() {
      self
        .progress_bar
        .set_message(format!("building {}", module.raw_request()));
    } else {
      self.progress_bar.set_message("building");
    }
    self.modules_count.fetch_add(1, SeqCst);
    Ok(())
  }

  async fn succeed_module(&self, _module: &dyn Module) -> Result<()> {
    let previous_modules_done = self.modules_done.fetch_add(1, SeqCst);
    let modules_done = previous_modules_done + 1;
    let percent = (modules_done as f32)
      / (cmp::max(
        self.last_modules_count.read().expect("TODO:").unwrap_or(1),
        self.modules_count.load(SeqCst),
      ) as f32);
    self
      .progress_bar
      .set_position((10.0 + 55.0 * percent) as u64);
    Ok(())
  }

  async fn optimize_chunks(
    &self,
    _ctx: PluginContext,
    _args: OptimizeChunksArgs<'_>,
  ) -> PluginOptimizeChunksOutput {
    self.progress_bar.set_position(80);
    self.progress_bar.set_message("optimizing chunks");
    Ok(())
  }

  async fn process_assets_stage_additional(
    &self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    self.progress_bar.set_position(90);
    self.progress_bar.set_message("processing assets");
    Ok(())
  }

  async fn done<'s, 'c>(
    &self,
    _ctx: PluginContext,
    _args: DoneArgs<'s, 'c>,
  ) -> PluginBuildEndHookOutput {
    self.progress_bar.set_message("done");
    self.progress_bar.finish();
    *self.last_modules_count.write().expect("TODO:") = Some(self.modules_count.load(SeqCst));
    Ok(())
  }
}
