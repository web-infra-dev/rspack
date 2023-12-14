use std::sync::atomic::Ordering::SeqCst;
use std::sync::RwLock;
use std::time::Duration;
use std::{cmp, sync::atomic::AtomicU32, time::Instant};

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use rspack_core::{
  BoxModule, Compilation, DoneArgs, FactorizeArgs, MakeParam, Module, NormalModuleCreateData,
  OptimizeChunksArgs, Plugin, PluginBuildEndHookOutput, PluginContext, PluginFactorizeHookOutput,
  PluginMakeHookOutput, PluginNormalModuleFactoryModuleHookOutput, PluginOptimizeChunksOutput,
  PluginProcessAssetsOutput, ProcessAssetsArgs,
};
use rspack_error::Result;

#[derive(Debug, Clone, Default)]
pub struct ProgressPluginOptions {
  // the prefix name of progress bar
  pub prefix: String,
  pub profile: bool,
}

#[derive(Debug)]
pub struct ProgressPlugin {
  pub options: ProgressPluginOptions,
  pub progress_bar: ProgressBar,
  pub dependencies_count: AtomicU32,
  pub dependencies_done: AtomicU32,
  pub modules_count: AtomicU32,
  pub modules_done: AtomicU32,
  pub last_modules_count: RwLock<Option<u32>>,
  pub last_dependencies_count: RwLock<Option<u32>>,
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
  pub fn new(options: ProgressPluginOptions) -> Self {
    let progress_bar = ProgressBar::with_draw_target(Some(100), ProgressDrawTarget::stdout());
    progress_bar.set_style(
      ProgressStyle::with_template(
        "● {prefix:.bold} {bar:25.green/white.dim} ({percent}%) {wide_msg:.dim}",
      )
      .expect("TODO:")
      .progress_chars("━━"),
    );

    Self {
      options,
      progress_bar,
      modules_count: AtomicU32::new(0),
      dependencies_count: AtomicU32::new(0),
      dependencies_done: AtomicU32::new(0),
      modules_done: AtomicU32::new(0),
      last_modules_count: RwLock::new(None),
      last_dependencies_count: RwLock::new(None),
      last_active_module: RwLock::new(None),
      last_state_info: RwLock::new(vec![]),
      last_update_time: RwLock::new(Instant::now()),
    }
  }
  pub fn update_throttled(&self) {
    if *self.last_update_time.read().expect("TODO:") + Duration::from_millis(50) < Instant::now() {
      self.update();
    }
    // *self.last_update_time.write().expect("TODO:") = Instant::now();
  }
  fn update(&self) {
    let modules_done = self.modules_done.load(SeqCst);
    let dependencies_done = self.dependencies_done.load(SeqCst);
    let percent_by_module = (modules_done as f32)
      / (cmp::max(
        self.last_modules_count.read().expect("TODO:").unwrap_or(1),
        self.modules_count.load(SeqCst),
      ) as f32);

    let percent_by_dependencies = (self.dependencies_done.load(SeqCst) as f32)
      / (cmp::max(
        self
          .last_dependencies_count
          .read()
          .expect("TODO:")
          .unwrap_or(1),
        self.dependencies_count.load(SeqCst),
      ) as f32);

    let percent = (percent_by_module + percent_by_dependencies) / 2.0;

    let mut items = vec![];
    let mut stat_items = vec![];
    {
      stat_items.push(format!(
        "{}/{} dependencies",
        dependencies_done,
        self.dependencies_count.load(SeqCst)
      ));
      stat_items.push(format!(
        "{}/{} modules",
        modules_done,
        self.modules_count.load(SeqCst)
      ));
      items.push(stat_items.join(" "));
      let last_active_module = self.last_active_module.read().expect("TODO:");
      if let Some(last_active_module) = last_active_module.clone() {
        items.push(last_active_module);
      }
    }

    self.handler(0.1 + percent * 0.55, String::from("building"), items);
    *self.last_update_time.write().expect("TODO:") = Instant::now();
  }
  pub fn handler(&self, percent: f32, msg: String, state_items: Vec<String>) {
    if self.options.profile {
      self.default_handler(percent, msg, state_items);
    } else {
      self.progress_bar_handler(percent, msg, state_items);
    }
  }
  fn default_handler(&self, percentage: f32, msg: String, items: Vec<String>) {
    let full_state = [vec![msg.clone()], items.clone()].concat();
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
        } else if i + 1 > full_state.len() || last_state_info[i].value != full_state[i] {
          let diff = (now - last_state_info[i].time).as_millis();
          let report_state = if i > 0 {
            last_state_info[i - 1].value.clone() + " > " + last_state_info[i].value.clone().as_str()
          } else {
            last_state_info[i].value.clone()
          };

          let mut color = "\x1b[32m";
          if diff > 10000 {
            color = "\x1b[31m"
          } else if diff > 1000 {
            color = "\x1b[33m"
          }
          println!(
            "{}{} {} ms {}\x1B[0m",
            color,
            " | ".repeat(i),
            diff,
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
      println!(
        "{}% {} {}",
        (percentage * 100.0) as u32,
        msg,
        items.join("")
      );
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
    _params: &mut Vec<MakeParam>,
  ) -> PluginMakeHookOutput {
    if !self.options.profile {
      self.progress_bar.reset();
      self.progress_bar.set_prefix(self.options.prefix.clone());
    }
    self.handler(0.01, String::from("make"), vec![]);
    self.modules_count.store(0, SeqCst);
    self.modules_done.store(0, SeqCst);
    Ok(())
  }

  async fn factorize(
    &self,
    _ctx: PluginContext,
    _args: FactorizeArgs<'_>,
  ) -> PluginFactorizeHookOutput {
    self.dependencies_count.fetch_add(1, SeqCst);
    if self.dependencies_count.load(SeqCst) < 50 || self.dependencies_count.load(SeqCst) % 100 == 0
    {
      self.update_throttled()
    };
    Ok(None)
  }

  async fn normal_module_factory_module(
    &self,
    _ctx: PluginContext,
    module: BoxModule,
    _args: &NormalModuleCreateData,
  ) -> PluginNormalModuleFactoryModuleHookOutput {
    self.dependencies_done.fetch_add(1, SeqCst);
    if self.dependencies_done.load(SeqCst) < 50 || self.dependencies_done.load(SeqCst) % 100 == 0 {
      self.update_throttled()
    };
    Ok(module)
  }

  async fn build_module(&self, module: &mut dyn Module) -> Result<()> {
    if let Some(module) = module.as_normal_module() {
      self
        .last_active_module
        .write()
        .expect("TODO:")
        .replace(module.id().to_string());
    }
    self.modules_count.fetch_add(1, SeqCst);
    self.update();
    Ok(())
  }

  async fn succeed_module(&self, module: &dyn Module) -> Result<()> {
    let id = module.identifier().to_string();
    self.modules_done.fetch_add(1, SeqCst);
    if self
      .last_active_module
      .read()
      .expect("TODO:")
      .as_ref()
      .is_some_and(|module| module.eq(&id))
    {
      self.update();
    } else if self.modules_done.load(SeqCst) < 50 || self.modules_done.load(SeqCst) % 100 == 0 {
      self.update_throttled();
    }
    Ok(())
  }

  // TODO: entries count

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
    if !self.options.profile {
      self.progress_bar.finish();
    }
    *self.last_modules_count.write().expect("TODO:") = Some(self.modules_count.load(SeqCst));
    *self.last_dependencies_count.write().expect("TODO:") =
      Some(self.dependencies_count.load(SeqCst));
    Ok(())
  }
}
