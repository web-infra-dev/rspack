use async_trait::async_trait;
use core::fmt::Debug;
use rspack_core::{BundleContext, Plugin, PluginLoadHookOutput};
use rspack_swc::swc_common::private::serde::de;
use std::ops::MulAssign;
pub static PLUGIN_NAME: &'static str = "rspack_progress";
// use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, Weak};
extern crate console;
use console::{style, Color, Term};
use rspack_core::{ast, BundleMode, SWC_GLOBALS};
use std::cell::RefCell;
use std::io;
use std::path::Path;
use std::thread;
use std::time::Duration;
use std::{env, fs};

fn get_bar(w: u16, current: u16, fe_color: u8, bg_color: u8) -> String {
  let fe = format!("{}", style(" ").bg(Color::Color256(fe_color)).to_string());
  let bg = format!("{}", style(" ").bg(Color::Color256(bg_color)).to_string());

  let mut s = String::from("");
  for _ in 0..current {
    s += &fe;
  }
  for _ in 0..w - current {
    s += &bg;
  }
  s
}
static CYAN: u8 = 51;
static GREY: u8 = 8;

fn clamp(n: u32, min: u32, max: u32) -> u32 {
  if n < min {
    return min;
  }
  if n > max {
    return max;
  }
  n
}

fn truncate(s: &str, max_chars: usize) -> String {
  let prefix = String::from("...");
  if max_chars >= s.len() + prefix.len() {
    return String::from(s);
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
  term: Arc<Mutex<Term>>,
  current: Arc<Mutex<u32>>,
  total: Arc<Mutex<u32>>,
}
impl ProgressBar {
  pub fn new(total: u32) -> ProgressBar {
    let current = Arc::new(Mutex::new(0));
    let total = Arc::new(Mutex::new(total));
    let term = Arc::new(Mutex::new(Term::stdout()));

    ProgressBar {
      total,
      current,
      term,
    }
  }

  pub fn update(&self, key: &str, filename: &str, delta: u32, clear: bool) -> io::Result<()> {
    let term = self.term.lock().unwrap();
    if clear {
      term.clear_screen()?;
      // term.clear_last_lines(1)?;
    } else {
      term.clear_screen()?;
      // term.clear_screen()?;
    }
    let mut x = self.current.lock().unwrap();
    *x = *x + delta;
    let t = *self.total.lock().unwrap();
    let c = *x;

    let mut s = style(key).cyan().to_string();
    s += " ";
    let frames: Vec<&str> = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    if t == 0 {
      s += &style(frames[(c % frames.len() as u32) as usize])
        .cyan()
        .to_string();
      s += " ";
    } else {
      let d = clamp(((c as f32) / (t as f32) * 100.0 / 4.0) as u32, 0, 25);
      s += &get_bar(25, d as u16, CYAN, GREY);
      let p = clamp((c as f32 / t as f32 * 100.0) as u32, 0, 100);
      s += &style(&format!(" {:3}% ", p)).cyan().to_string();
    }

    s += &style(&truncate(filename, 25)).dim().to_string();
    s += "\n";
    term.write_str(&s)?;
    Ok(())
  }

  pub fn finish_and_clear(&self) -> io::Result<()> {
    Ok(())
  }
}

#[derive(Debug)]
pub struct ProgressPlugin {
  progress: ProgressBar,
}
impl ProgressPlugin {
  pub fn new() -> ProgressPlugin {
    let progress = ProgressBar::new(25);
    progress.update("RsPack", "", 0, false).unwrap();
    ProgressPlugin { progress }
  }
}
use md5;
use std::fs::File;
use std::io::Write;
#[async_trait]
impl Plugin for ProgressPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  async fn build_start(&self, _ctx: &BundleContext) {
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
        .unwrap();
    }
    *self.progress.total.lock().unwrap() = total;
  }

  async fn load(&self, _ctx: &BundleContext, id: &str) -> PluginLoadHookOutput {
    // if matches!(_ctx.options.mode, BundleMode::Dev) {
    //   return None;
    // }
    self.progress.update("RsPack", id, 1, true).unwrap();
    None
  }
  async fn build_end(&self, _ctx: &BundleContext) {
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
  }
}
