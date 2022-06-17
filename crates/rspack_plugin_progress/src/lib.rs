#![deny(clippy::all)]

use anyhow::Result;
use async_trait::async_trait;
use console::{style, Color, StyledObject, Term};
use core::fmt::Debug;
use rspack_core::{
  Asset, Plugin, PluginBuildEndHookOutput, PluginBuildStartHookOutput, PluginContext,
};
use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;
use std::fs::File;
use std::io::Write;
use std::{env, fs};

pub static PLUGIN_NAME: &str = "rspack_progress";
// color256 https://www.ditig.com/256-colors-cheat-sheet
static CYAN: Color = Color::Color256(51);
static GREEN: Color = Color::Color256(2);
static GREY: Color = Color::Color256(8);
static YELLOW: Color = Color::Color256(178);
static RED: Color = Color::Color256(9);
static MAGENTA: Color = Color::Color256(201);
static BLUE: Color = Color::Color256(12);

static TERM: Lazy<Term> = Lazy::new(Term::buffered_stdout);

static MAX_BAR_WIDTH: usize = 25;
static MAX_TEXT_WIDTH: usize = 30;
static FRAMES: &[&str; 10] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

fn get_bar_str(
  key: &str,
  total: u32,
  current: u32,
  text: &str,
  fe_color: Color,
  bg_color: Color,
) -> String {
  let mut s = style(format!(" {} ◯ ", key)).fg(fe_color).to_string();
  if total == 0 {
    s += &style(FRAMES[(current % FRAMES.len() as u32) as usize])
      .fg(fe_color)
      .to_string();
    s += " ";
  } else {
    let left = (((current as f32) / (total as f32) * MAX_BAR_WIDTH as f32).round() as usize)
      .clamp(0, MAX_BAR_WIDTH);
    s += &style(" ".repeat(left)).bg(fe_color).to_string();
    s += &style(" ".repeat(MAX_BAR_WIDTH - left))
      .bg(bg_color)
      .to_string();
  }

  let percent = ((current as f32 / total as f32 * 100.0) as u32).clamp(0, 100);
  s += &style(&format!(" {:3}% ", percent)).fg(fe_color).to_string();
  s += &style(&truncate(text, MAX_TEXT_WIDTH)).dim().to_string();
  let term_width = TERM.size().1 as usize;
  let line_width = console::measure_text_width(&s);
  s += &" ".repeat(term_width.saturating_sub(line_width));
  s
}

fn truncate(s: &str, max_chars: usize) -> String {
  let prefix = String::from("...");
  if max_chars >= s.len() + prefix.len() {
    return format!("{:>width$}", s, width = max_chars);
  }
  match s
    .char_indices()
    .nth(s.len() - 1 - (max_chars - prefix.len()))
  {
    None => (prefix + s),
    Some((idx, _)) => prefix + &s[idx + 1..],
  }
}

#[derive(Debug, Clone)]
pub struct ProgressBar {
  current: Arc<Mutex<u32>>,
  total: Arc<Mutex<u32>>,
  done: Arc<Mutex<bool>>,
}

impl Default for ProgressBar {
  fn default() -> Self {
    Self::new()
  }
}

impl ProgressBar {
  pub fn new() -> ProgressBar {
    let current = Arc::new(Mutex::new(0));
    let total = Arc::new(Mutex::new(0));
    let done = Arc::new(Mutex::new(false));

    ProgressBar {
      total,
      current,
      done,
    }
  }

  pub fn update(&self, key: &str, filename: &str, delta: u32) -> Result<()> {
    let mut current = self
      .current
      .lock()
      .map_err(|_| anyhow::anyhow!("failed to acquire lock of current"))?;
    *current += delta;
    let total = self
      .total
      .lock()
      .map_err(|_| anyhow::anyhow!("failed to acquire lock of total"))?;
    TERM.clear_line()?;
    let s = get_bar_str(key, *total, *current, filename, GREEN, GREY);
    TERM.write_str(&s)?;
    TERM.flush()?;
    Ok(())
  }

  pub fn finish_and_clear(&self) -> Result<()> {
    TERM.clear_line()?;
    let total = *self
      .current
      .lock()
      .map_err(|_| anyhow::anyhow!("failed to acquire lock of current"))?;
    let s = format!(
      "{}{} files transformed. \n",
      style("finished ✔ ").green(),
      total
    );
    TERM.write_str(&s)?;
    TERM.flush()?;
    Ok(())
  }
}
#[derive(Debug)]
pub struct ProgressPlugin {
  progress: ProgressBar,
}
impl ProgressPlugin {
  pub fn new() -> ProgressPlugin {
    let progress = ProgressBar::new();
    ProgressPlugin { progress }
  }
}

impl Default for ProgressPlugin {
  fn default() -> ProgressPlugin {
    Self::new()
  }
}

#[async_trait]
impl Plugin for ProgressPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  #[inline]
  fn need_resolve(&self) -> bool {
    false
  }

  #[inline]
  fn need_load(&self) -> bool {
    false
  }

  #[inline]
  fn need_tap_generated_chunk(&self) -> bool {
    false
  }
  async fn build_start(&self, _ctx: &PluginContext) -> PluginBuildStartHookOutput {
    // if matches!(_ctx.options.mode, BundleMode::Dev) {
    //   return;
    // }
    let hash = format!("{:x}", md5::compute(_ctx.options.root.clone()));
    let file_path = env::temp_dir().join(hash);
    let mut total = 0;
    if file_path.exists() {
      total = fs::read_to_string(file_path)
        .unwrap()
        .trim()
        .parse()
        .unwrap_or(0);
    }
    *self.progress.total.lock().unwrap() = total;

    Ok(())
  }
  fn module_parsed(&self, _ctx: &PluginContext, uri: &str) -> Result<()> {
    let done = self
      .progress
      .done
      .lock()
      .map_err(|_| anyhow::anyhow!("failed to acquire lock of done"))?;
    if *done {
      return Ok(());
    }
    // if matches!(_ctx.options.mode, BundleMode::Dev) {
    //   return None;
    // }
    self.progress.update("RsPack", uri, 1)?;
    Ok(())
  }
  async fn build_end(&self, _ctx: &PluginContext, _asset: &[Asset]) -> PluginBuildEndHookOutput {
    let mut done = self.progress.done.lock().unwrap();
    if *done {
      return Ok(());
    }
    *done = true;
    // if matches!(_ctx.options.mode, BundleMode::Dev) {
    //   return;
    // }
    self.progress.finish_and_clear().unwrap();
    let hash = format!("{:x}", md5::compute(_ctx.options.root.clone()));
    let file_path = env::temp_dir().join(hash);
    let mut file = File::create(file_path).unwrap();
    let total = *self.progress.current.lock().unwrap();
    let s = total.to_string();
    file.write_all(s.as_bytes()).unwrap();

    let assets = _ctx.assets().lock().unwrap();

    let outdir = _ctx
      .options
      .outdir
      .split('/')
      .last()
      .unwrap_or("dist")
      .to_string()
      + "/";
    let outdir = style(outdir).fg(GREY).to_string();
    let mut max_name_len = 0;
    let mut asset_list: Vec<(String, usize)> = vec![]; // (name, size)
    assets.iter().for_each(|asset| {
      let name: String = if asset.filename.starts_with('/') {
        (&asset.filename[1..]).replace(&_ctx.options.outdir, "")
      } else {
        asset.filename.clone()
      };
      max_name_len = max_name_len.max(name.len());
      let size = asset.source.len();
      asset_list.push((name, size));
    });

    asset_list.sort_unstable_by(|a, b| a.1.cmp(&b.1));

    for (name, size) in asset_list.iter() {
      let ext = name.split('.').last().unwrap_or("");
      let size = size_with_color(*size);
      let color = guess_color_by_ext(ext);
      let name = style(format!("{:width$}", name, width = max_name_len)).fg(color);

      println!("{}{}    {}", outdir, name, size);
    }

    Ok(())
  }
}

fn guess_color_by_ext(ext: &str) -> Color {
  match ext {
    "js" => CYAN,
    "css" => MAGENTA,
    "asset" => GREEN,
    "html" => BLUE,
    "map" => GREY,
    _ => GREY,
  }
}
fn size_with_color(n: usize) -> StyledObject<String> {
  let kb = 1_000;
  let mb = 1_000_000;
  let gb = 1_000_000_000;
  let warn_limit = 200.0;
  let danger_limit = 500.0;

  if n < kb {
    style(format!("{n:6.2} B")).fg(GREY);
  } else if n < mb {
    let n = (n as f64) / (kb) as f64;
    let color = if n < warn_limit {
      GREY
    } else if n < danger_limit {
      YELLOW
    } else {
      RED
    };
    style(format!("{n:6.2} K")).fg(color);
  } else if n < gb {
    let n = (n as f64) / (mb) as f64;
    style(format!("{n:6.2} M")).fg(RED);
  }
  style(format!("{n:6.2} B")).fg(GREY)
}
