use std::cmp::Ordering;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::RwLock;
use std::time::Duration;
use std::{cmp, sync::atomic::AtomicU32, time::Instant};

use async_trait::async_trait;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use linked_hash_map::LinkedHashMap as HashMap;
use rspack_core::{
  ApplyContext, BoxModule, Compilation, CompilationParams, CompilerOptions, DoneArgs, MakeParam,
  ModuleIdentifier, OptimizeChunksArgs, Plugin, PluginBuildEndHookOutput, PluginContext,
  PluginOptimizeChunksOutput,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook, AsyncSeries, AsyncSeries2};

#[derive(Debug, Clone, Default)]
pub struct ProgressPluginOptions {
  // the prefix name of progress bar
  pub prefix: String,
  pub profile: bool,
}

#[derive(Debug)]
pub struct ProgressPluginStateInfo {
  pub value: String,
  pub time: Instant,
  pub duration: Option<Duration>,
}

#[plugin]
#[derive(Debug)]
pub struct ProgressPlugin {
  pub options: ProgressPluginOptions,
  pub progress_bar: ProgressBar,
  pub modules_count: AtomicU32,
  pub modules_done: AtomicU32,
  pub active_modules: RwLock<HashMap<ModuleIdentifier, Instant>>,
  pub last_modules_count: RwLock<Option<u32>>,
  pub last_active_module: RwLock<Option<ModuleIdentifier>>,
  pub last_state_info: RwLock<Vec<ProgressPluginStateInfo>>,
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

    Self::new_inner(
      options,
      progress_bar,
      AtomicU32::new(0),
      AtomicU32::new(0),
      Default::default(),
      Default::default(),
      Default::default(),
      Default::default(),
    )
  }

  fn update(&self) {
    let modules_done = self.modules_done.load(Relaxed);
    let percent_by_module = (modules_done as f32)
      / (cmp::max(
        self.last_modules_count.read().expect("TODO:").unwrap_or(1),
        self.modules_count.load(Relaxed),
      ) as f32);

    let mut items = vec![];
    let last_active_module = self.last_active_module.read().expect("TODO:");

    if let Some(last_active_module) = *last_active_module {
      items.push(last_active_module.to_string());
      let duration = self
        .active_modules
        .read()
        .expect("TODO:")
        .get(&last_active_module)
        .map(|time| Instant::now() - *time);
      self.handler(
        0.1 + percent_by_module * 0.55,
        String::from("building"),
        items,
        duration,
      );
    }
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

  fn default_handler(&self, _: f32, msg: String, items: Vec<String>, duration: Option<Duration>) {
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
              duration: None,
            },
          )
        } else if i + 1 > full_state.len() || !last_state_info[i].value.eq(&full_state[i]) {
          let diff = match last_state_info[i].duration {
            Some(duration) => duration,
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
          match (i + 1).cmp(&full_state.len()) {
            Ordering::Greater => last_state_info.truncate(i),
            Ordering::Equal => {
              last_state_info[i] = ProgressPluginStateInfo {
                value: full_state[i].clone(),
                time: now,
                duration,
              }
            }
            Ordering::Less => {
              last_state_info[i] = ProgressPluginStateInfo {
                value: full_state[i].clone(),
                time: now,
                duration: None,
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

#[plugin_hook(AsyncSeries2<Compilation, CompilationParams> for ProgressPlugin)]
async fn this_compilation(
  &self,
  _compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  self.handler(
    0.08,
    "setup".to_string(),
    vec!["compilation".to_string()],
    None,
  );
  Ok(())
}

#[plugin_hook(AsyncSeries2<Compilation, CompilationParams> for ProgressPlugin)]
async fn compilation(
  &self,
  _compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  self.handler(
    0.09,
    "setup".to_string(),
    vec!["compilation".to_string()],
    None,
  );
  Ok(())
}

#[plugin_hook(AsyncSeries2<Compilation, Vec<MakeParam>> for ProgressPlugin)]
async fn make(&self, _compilation: &mut Compilation, _params: &mut Vec<MakeParam>) -> Result<()> {
  if !self.options.profile {
    self.progress_bar.reset();
    self.progress_bar.set_prefix(self.options.prefix.clone());
  }
  self.handler(0.01, String::from("make"), vec![], None);
  self.modules_count.store(0, Relaxed);
  self.modules_done.store(0, Relaxed);
  Ok(())
}

#[plugin_hook(AsyncSeries<BoxModule> for ProgressPlugin)]
async fn build_module(&self, module: &mut BoxModule) -> Result<()> {
  self
    .active_modules
    .write()
    .expect("TODO:")
    .insert(module.identifier(), Instant::now());
  self.modules_count.fetch_add(1, Relaxed);
  self
    .last_active_module
    .write()
    .expect("TODO:")
    .replace(module.identifier());
  if !self.options.profile {
    self.update();
  }
  Ok(())
}

#[plugin_hook(AsyncSeries<BoxModule> for ProgressPlugin)]
async fn succeed_module(&self, module: &mut BoxModule) -> Result<()> {
  self.modules_done.fetch_add(1, Relaxed);
  self
    .last_active_module
    .write()
    .expect("TODO:")
    .replace(module.identifier());

  // only profile mode should update at succeed module
  if self.options.profile {
    self.update();
  }
  let mut last_active_module = Default::default();
  {
    let mut active_modules = self.active_modules.write().expect("TODO:");
    active_modules.remove(&module.identifier());

    // get the last active module
    if !self.options.profile {
      active_modules.iter().for_each(|(module, _)| {
        last_active_module = *module;
      });
    }
  }
  if !self.options.profile {
    self
      .last_active_module
      .write()
      .expect("TODO:")
      .replace(last_active_module);
    if !last_active_module.is_empty() {
      self.update();
    }
  }
  Ok(())
}

#[plugin_hook(AsyncSeries<Compilation> for ProgressPlugin)]
async fn finish_make(&self, _compilation: &mut Compilation) -> Result<()> {
  self.handler(
    0.69,
    "building".to_string(),
    vec!["finish make".to_string()],
    None,
  );
  Ok(())
}

#[plugin_hook(AsyncSeries<Compilation> for ProgressPlugin)]
async fn finish_modules(&self, _compilation: &mut Compilation) -> Result<()> {
  self.sealing_hooks_report("finish modules", 0);
  Ok(())
}

#[plugin_hook(AsyncSeries<Compilation> for ProgressPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_ADDITIONAL)]
async fn process_assets(&self, _compilation: &mut Compilation) -> Result<()> {
  self.sealing_hooks_report("asset processing", 35);
  Ok(())
}

#[plugin_hook(AsyncSeries<Compilation> for ProgressPlugin)]
async fn after_process_assets(&self, _compilation: &mut Compilation) -> Result<()> {
  self.sealing_hooks_report("after asset optimization", 36);
  Ok(())
}

#[plugin_hook(AsyncSeries<Compilation> for ProgressPlugin)]
async fn emit(&self, _compilation: &mut Compilation) -> Result<()> {
  self.handler(0.98, "emitting".to_string(), vec!["emit".to_string()], None);
  Ok(())
}

#[plugin_hook(AsyncSeries<Compilation> for ProgressPlugin)]
async fn after_emit(&self, _compilation: &mut Compilation) -> Result<()> {
  self.handler(
    0.98,
    "emitting".to_string(),
    vec!["after emit".to_string()],
    None,
  );
  Ok(())
}

#[async_trait]
impl Plugin for ProgressPlugin {
  fn name(&self) -> &'static str {
    "progress"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .this_compilation
      .tap(this_compilation::new(self));
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));
    ctx.context.compiler_hooks.make.tap(make::new(self));
    ctx
      .context
      .compilation_hooks
      .build_module
      .tap(build_module::new(self));
    ctx
      .context
      .compilation_hooks
      .succeed_module
      .tap(succeed_module::new(self));
    ctx
      .context
      .compiler_hooks
      .finish_make
      .tap(finish_make::new(self));
    ctx
      .context
      .compilation_hooks
      .finish_modules
      .tap(finish_modules::new(self));
    ctx
      .context
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    ctx
      .context
      .compilation_hooks
      .after_process_assets
      .tap(after_process_assets::new(self));
    ctx.context.compiler_hooks.emit.tap(emit::new(self));
    ctx
      .context
      .compiler_hooks
      .after_emit
      .tap(after_emit::new(self));
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

  async fn done<'s, 'c>(
    &self,
    _ctx: PluginContext,
    _args: DoneArgs<'s, 'c>,
  ) -> PluginBuildEndHookOutput {
    self.handler(1.0, String::from("done"), vec![], None);
    if !self.options.profile {
      self.progress_bar.finish();
    }
    *self.last_modules_count.write().expect("TODO:") = Some(self.modules_count.load(Relaxed));
    Ok(())
  }
}
