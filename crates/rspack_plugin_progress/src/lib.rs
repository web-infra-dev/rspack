use std::sync::atomic::Ordering::SeqCst;
use std::sync::RwLock;
use std::time::Duration;
use std::{cmp, sync::atomic::AtomicU32, time::Instant};

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use rspack_core::{
  BoxModule, Compilation, CompilationArgs, CompilationParams, DoneArgs, MakeParam, Module,
  ModuleIdentifier, NormalModuleCreateData, OptimizeChunksArgs, Plugin, PluginBuildEndHookOutput,
  PluginCompilationHookOutput, PluginContext, PluginMakeHookOutput,
  PluginNormalModuleFactoryModuleHookOutput, PluginOptimizeChunksOutput, PluginProcessAssetsOutput,
  PluginThisCompilationHookOutput, ProcessAssetsArgs, ThisCompilationArgs,
};
use rspack_error::Result;
use rustc_hash::FxHashMap as HashMap;

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
  pub active_modules: RwLock<HashMap<ModuleIdentifier, Instant>>,
  pub last_modules_count: RwLock<Option<u32>>,
  pub last_dependencies_count: RwLock<Option<u32>>,
  pub last_active_module: RwLock<Option<ModuleIdentifier>>,
  pub last_state_info: RwLock<Vec<ProgressPluginStateInfo>>,
  pub last_update_time: RwLock<Instant>,
  pub sealing_hooks_report_index: AtomicU32,
  pub number_of_sealing_hooks: u32,
  pub queue: Vec<String>,
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
      active_modules: RwLock::new(HashMap::default()),
      last_modules_count: RwLock::new(None),
      last_dependencies_count: RwLock::new(None),
      last_active_module: RwLock::new(None),
      last_state_info: RwLock::new(vec![]),
      last_update_time: RwLock::new(Instant::now()),
      sealing_hooks_report_index: AtomicU32::new(0),
      number_of_sealing_hooks: 20,
      queue: vec![],
    }
  }
  pub fn update_throttled(&self) {
    if *self.last_update_time.read().expect("TODO:") + Duration::from_millis(50) < Instant::now() {
      self.update(None);
    }
    // *self.last_update_time.write().expect("TODO:") = Instant::now();
  }
  fn update(&self, time: Option<Duration>) {
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
    let mut stat_items: Vec<String> = vec![];
    {
      // TODO: dependencies_done modules_done report
      // stat_items.push(format!(
      //   "{}/{} dependencies",
      //   dependencies_done,
      //   self.dependencies_count.load(SeqCst)
      // ));
      // stat_items.push(format!(
      //   "{}/{} modules",
      //   modules_done,
      //   self.modules_count.load(SeqCst)
      // ));
      // stat_items.push(format!("{} active", self.active_modules.read().expect("TODO:").len()));
      // items.push(stat_items.join(" "));
      let last_active_module = self.last_active_module.read().expect("TODO:");

      if let Some(last_active_module) = last_active_module.clone() {
        items.push(last_active_module.to_string());
      }
    }

    self.handler(0.1 + percent * 0.55, String::from("building"), items, time);
    // self.queue.push() * self.last_update_time.write().await = Instant::now();
  }
  pub fn handler(
    &self,
    percent: f32,
    msg: String,
    state_items: Vec<String>,
    time: Option<Duration>,
  ) {
    if self.options.profile {
      self.default_handler(percent, msg, state_items, time);
    } else {
      self.progress_bar_handler(percent, msg, state_items);
    }
  }
  fn default_handler(
    &self,
    percentage: f32,
    msg: String,
    items: Vec<String>,
    time: Option<Duration>,
  ) {
    let full_state = [vec![msg.clone()], items.clone()].concat();
    let now = Instant::now();
    {
      let mut last_state_info = self.last_state_info.write().expect("TODO:");
      // println!("{:?} {:?}", full_state, last_state_info);
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
        } else if i + 1 > full_state.len() || !last_state_info[i].value.eq(&full_state[i]) {
          let diff = match time {
            Some(time) => time,
            _ => now - last_state_info[i].time,
          }
          .as_millis();
          let report_state = if i > 0 {
            last_state_info[i - 1].value.clone() + " > " + last_state_info[i].value.clone().as_str()
          } else {
            last_state_info[i].value.clone()
          };

          if diff > 5 {
            // TODO: color map
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
          }

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
  fn progress_bar_handler(&self, percent: f32, msg: String, state_items: Vec<String>) {
    self
      .progress_bar
      .set_message(msg + " " + state_items.join(" ").as_str());
    self.progress_bar.set_position((percent * 100.0) as u64);
  }

  fn sealing_hooks_report(&self, name: &str, index: i32) {
    let number_of_sealing_hooks = 38;
    self.handler(
      0.7 + 0.25 * (index / number_of_sealing_hooks) as f32,
      "sealing".to_string(),
      vec![name.to_string()],
      None,
    );
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
    self.handler(0.01, String::from("make"), vec![], None);
    self.modules_count.store(0, SeqCst);
    self.modules_done.store(0, SeqCst);
    Ok(())
  }

  async fn before_compile(&self, _params: &CompilationParams) -> Result<()> {
    self.handler(
      0.06,
      "setup".to_string(),
      vec!["before compile".to_string()],
      None,
    );
    Ok(())
  }

  async fn this_compilation(
    &self,
    _args: ThisCompilationArgs<'_>,
    _params: &CompilationParams,
  ) -> PluginThisCompilationHookOutput {
    self.handler(
      0.08,
      "setup".to_string(),
      vec!["compilation".to_string()],
      None,
    );
    Ok(())
  }

  async fn compilation(
    &self,
    _args: CompilationArgs<'_>,
    _params: &CompilationParams,
  ) -> PluginCompilationHookOutput {
    self.handler(
      0.09,
      "setup".to_string(),
      vec!["compilation".to_string()],
      None,
    );
    Ok(())
  }

  // async fn factorize(
  //   &self,
  //   _ctx: PluginContext,
  //   _args: FactorizeArgs<'_>,
  // ) -> PluginFactorizeHookOutput {
  //   self.dependencies_count.fetch_add(1, SeqCst);
  //   if self.dependencies_count.load(SeqCst) < 50 || self.dependencies_count.load(SeqCst) % 100 == 0
  //   {
  //     self.update_throttled().await
  //   };
  //   Ok(None)
  // }

  // async fn normal_module_factory_module(
  //   &self,
  //   _ctx: PluginContext,
  //   module: BoxModule,
  //   _args: &NormalModuleCreateData,
  // ) -> PluginNormalModuleFactoryModuleHookOutput {
  //   self.dependencies_done.fetch_add(1, SeqCst);
  //   if self.dependencies_done.load(SeqCst) < 50 || self.dependencies_done.load(SeqCst) % 100 == 0 {
  //     self.update_throttled().await
  //   };
  //   Ok(module)
  // }

  async fn build_module(&self, module: &mut dyn Module) -> Result<()> {
    self
      .active_modules
      .write()
      .expect("TODO:")
      .insert(module.identifier(), Instant::now());
    self.modules_count.fetch_add(1, SeqCst);
    // self.update().await;
    Ok(())
  }

  async fn succeed_module(&self, module: &dyn Module) -> Result<()> {
    self.modules_done.fetch_add(1, SeqCst);
    // let mut last_active_module = self.last_active_module.write().expect("TODO:");
    // let mut last_active_module = String::new();
    // active_modules.iter().for_each(|module| {
    //   last_active_module = module.to_string();
    // });

    // if self
    //   .last_active_module
    //   .read()
    //   .expect("TODO:")
    //   .as_ref()
    //   .is_some_and(|module| module.eq(&id))
    // {
    self
      .last_active_module
      .write()
      .expect("TODO:")
      .replace(module.identifier());

    let time = self
      .active_modules
      .read()
      .expect("TODO:")
      .get(&module.identifier())
      .map(|time| Instant::now() - *time);
    self.update(time);
    // } else if self.modules_done.load(SeqCst) < 50 || self.modules_done.load(SeqCst) % 100 == 0 {
    //   self.update_throttled();
    // }
    let mut active_modules = self.active_modules.write().expect("TODO:");
    active_modules.remove(&module.identifier());
    Ok(())
  }

  async fn finish_make(&self, _compilation: &mut Compilation) -> Result<()> {
    self.handler(
      0.69,
      "building".to_string(),
      vec!["finish make".to_string()],
      None,
    );
    Ok(())
  }

  async fn finish_modules(&self, _modules: &mut Compilation) -> Result<()> {
    self.sealing_hooks_report("finish modules", 0);
    Ok(())
  }

  fn seal(&self, _compilation: &mut Compilation) -> Result<()> {
    self.sealing_hooks_report("plugins", 1);
    Ok(())
  }

  async fn optimize_dependencies(&self, _compilation: &mut Compilation) -> Result<Option<()>> {
    self.sealing_hooks_report("dependencies", 2);
    Ok(None)
  }

  async fn optimize_modules(&self, _compilation: &mut Compilation) -> Result<()> {
    self.sealing_hooks_report("module optimization", 7);
    Ok(())
  }

  async fn after_optimize_modules(&self, _compilation: &mut Compilation) -> Result<()> {
    self.sealing_hooks_report("after module optimization", 8);
    Ok(())
  }

  async fn optimize_chunks(
    &self,
    _ctx: PluginContext,
    _args: OptimizeChunksArgs<'_>,
  ) -> PluginOptimizeChunksOutput {
    self.sealing_hooks_report("chunk optimization", 9);
    Ok(())
  }

  async fn optimize_tree(&self, _compilation: &mut Compilation) -> Result<()> {
    self.sealing_hooks_report("module and chunk tree optimization", 11);
    Ok(())
  }

  async fn optimize_chunk_modules(&self, _args: OptimizeChunksArgs<'_>) -> Result<()> {
    self.sealing_hooks_report("chunk modules optimization", 13);
    Ok(())
  }

  fn module_ids(&self, _modules: &mut Compilation) -> Result<()> {
    self.sealing_hooks_report("module ids", 16);
    Ok(())
  }

  fn chunk_ids(&self, _compilation: &mut Compilation) -> Result<()> {
    self.sealing_hooks_report("chunk ids", 21);
    Ok(())
  }

  async fn process_assets_stage_additional(
    &self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    self.sealing_hooks_report("asset processing", 35);
    Ok(())
  }

  async fn after_process_assets(
    &self,
    _ctx: PluginContext,
    _args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    self.sealing_hooks_report("after asset optimization", 36);
    Ok(())
  }

  async fn emit(&self, _compilation: &mut Compilation) -> Result<()> {
    self.handler(0.98, "emitting".to_string(), vec!["emit".to_string()], None);
    Ok(())
  }

  async fn after_emit(&self, _compilation: &mut Compilation) -> Result<()> {
    self.handler(
      0.98,
      "emitting".to_string(),
      vec!["after emit".to_string()],
      None,
    );
    Ok(())
  }

  async fn done<'s, 'c>(
    &self,
    _ctx: PluginContext,
    _args: DoneArgs<'s, 'c>,
  ) -> PluginBuildEndHookOutput {
    self.handler(1.0, String::from("done"), vec![], None);
    self.handler(1.0, String::from(""), vec![], None);
    if !self.options.profile {
      self.progress_bar.finish();
    }
    *self.last_modules_count.write().expect("TODO:") = Some(self.modules_count.load(SeqCst));
    *self.last_dependencies_count.write().expect("TODO:") =
      Some(self.dependencies_count.load(SeqCst));
    Ok(())
  }
}
