use std::sync::atomic::Ordering::SeqCst;
use std::sync::RwLock;
use std::time::Duration;
use std::{cmp, sync::atomic::AtomicU32, time::Instant};

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
  pub profile: Option<bool>,
}

#[derive(Debug)]
pub struct ProgressPlugin {
  pub options: ProgressPluginConfig,
  pub progress_bar: ProgressBar,
  pub modules_count: AtomicU32,
  pub modules_done: AtomicU32,
  pub last_modules_count: RwLock<Option<u32>>,
  pub last_active_module: RwLock<Option<String>>,
  pub last_state_info: RwLock<Vec<ProgressPluginStateInfo>>,
  pub last_update_time: RwLock<Instant>,
}
#[derive(Debug)]
pub struct ProgressPluginStateInfo {
  pub value: String,
  pub time: Instant,
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
      last_active_module: RwLock::new(None),
      last_state_info: RwLock::new(vec![]),
      last_update_time: RwLock::new(Instant::now()),
    }
  }
  pub fn update_throttled(&self) {
    if *self.last_update_time.read().expect("TODO:") + Duration::from_millis(50) < Instant::now() {
      self.update();
    }
  }
  fn update(&self) {
    let previous_modules_done = self.modules_done.fetch_add(1, SeqCst);
    let modules_done = previous_modules_done + 1;
    let percent = (modules_done as f32)
      / (cmp::max(
        self.last_modules_count.read().expect("TODO:").unwrap_or(1),
        self.modules_count.load(SeqCst),
      ) as f32);
    let mut state_items = vec![];
    {
      let last_active_module = self.last_active_module.read().expect("TODO:");
      if let Some(last_active_module) = last_active_module.clone() {
        state_items.push(last_active_module.clone());
      }
    }
    self.handler(0.1 + percent * 0.55, String::from("building"), state_items);
    *self.last_update_time.write().expect("TODO:") = Instant::now();
  }
  pub fn handler(&self, percent: f32, msg: String, state_items: Vec<String>) {
    if self.options.profile.unwrap_or(false) {
      self.default_handler(percent, msg, state_items);
    } else {
      self.progress_bar_handler(percent, msg, state_items);
    }
  }
  fn default_handler(&self, percent: f32, msg: String, state_items: Vec<String>) {
    let full_state = [vec![msg.clone()], state_items].concat();
    let now = Instant::now();
    {
      let mut last_state_info = self.last_state_info.write().expect("TODO:");

      let len = full_state.len().max(last_state_info.len());
      let original_last_state_info_len = last_state_info.len();
      for i in (0..len).rev() {
        if i + 1 > original_last_state_info_len {
          last_state_info.insert(
            original_last_state_info_len,
            ProgressPluginStateInfo {
              value: full_state[i].clone(),
              time: now,
            },
          )
        } else {
          if i + 1 > full_state.len() || last_state_info[i].value != full_state[i] {
            let diff = now - last_state_info[i].time;
            let report_state = if i > 0 {
              last_state_info[i - 1].value.clone()
                + " > "
                + last_state_info[i].value.clone().as_str()
            } else {
              last_state_info[i].value.clone()
            };
            println!(
              "{} {}ms {}",
              " | ".repeat(i),
              diff.as_millis(),
              report_state
            );
            if i + 1 > full_state.len() {
              last_state_info.truncate(i);
            } else {
              last_state_info[i] = ProgressPluginStateInfo {
                value: full_state[i].clone(),
                time: now,
              };
            }
          }
        }
      }
    }
  }
  fn progress_bar_handler(&self, percent: f32, msg: String, state_items: Vec<String>) {
    self
      .progress_bar
      .set_message(msg + " " + state_items.join(" ").as_str());
    self.progress_bar.set_position((percent * 100.0) as u64);
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
    if self.options.profile.unwrap_or(false) == false {
      self.progress_bar.reset();
      self.progress_bar.set_prefix(
        self
          .options
          .prefix
          .clone()
          .unwrap_or_else(|| "Rspack".to_string()),
      );
    }
    self.handler(0.01, String::from("make"), vec![]);
    self.modules_count.store(0, SeqCst);
    self.modules_done.store(0, SeqCst);
    Ok(())
  }

  async fn build_module(&self, module: &mut dyn Module) -> Result<()> {
    if let Some(module) = module.as_normal_module() {
      self
        .last_active_module
        .write()
        .expect("TODO:")
        .replace(module.raw_request().to_string());
    }
    self.modules_count.fetch_add(1, SeqCst);
    self.update_throttled();
    Ok(())
  }
  async fn optimize_chunks(
    &self,
    _ctx: PluginContext,
    _args: OptimizeChunksArgs<'_>,
  ) -> PluginOptimizeChunksOutput {
    self.handler(0.8, String::from("optimizing chunks"), vec![]);
    Ok(())
  }

  async fn process_assets_stage_additional(
    &self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    self.handler(0.9, String::from("processing assets"), vec![]);

    Ok(())
  }

  async fn done<'s, 'c>(
    &self,
    _ctx: PluginContext,
    _args: DoneArgs<'s, 'c>,
  ) -> PluginBuildEndHookOutput {
    self.handler(1.0, String::from("done"), vec![]);
    if self.options.profile.unwrap_or(false) == false {
      self.progress_bar.finish();
    }
    *self.last_modules_count.write().expect("TODO:") = Some(self.modules_count.load(SeqCst));
    Ok(())
  }
}
