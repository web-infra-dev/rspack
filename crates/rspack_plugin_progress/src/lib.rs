use async_trait::async_trait;
use core::fmt::Debug;
use rspack_core::{BundleContext, LoadArgs, Plugin, PluginLoadHookOutput};
use std::sync::{Arc, Mutex};
extern crate console;
use console::{style, Color, Term};
use md5;
use once_cell::sync::Lazy;
use std::fs::File;
use std::io;
use std::io::Write;
use std::{env, fs};

pub static PLUGIN_NAME: &'static str = "rspack_progress";
// color256 https://www.ditig.com/256-colors-cheat-sheet
static CYAN: u8 = 51;
static GREEN: u8 = 2;
static GREY: u8 = 8;

static TERM: Lazy<Term> = Lazy::new(|| {
  let term = Term::buffered_stdout();
  term
});

static MAX_BAR_WIDTH: usize = 25;
static MAX_TEXT_WIDTH: usize = 30;
static FRAMES: &'static [&str; 10] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

fn get_bar_str(
  key: &str,
  total: u32,
  current: u32,
  text: &str,
  fe_color: u8,
  bg_color: u8,
) -> String {
  let mut s = style(format!(" {} ◯ ", key))
    .fg(Color::Color256(fe_color))
    .to_string();
  if total == 0 {
    s += &style(FRAMES[(current % FRAMES.len() as u32) as usize])
      .fg(Color::Color256(fe_color))
      .to_string();
    s += " ";
  } else {
    let left = (((current as f32) / (total as f32) * MAX_BAR_WIDTH as f32).round() as usize)
      .clamp(0, MAX_BAR_WIDTH);
    s += &style(" ".repeat(left))
      .bg(Color::Color256(fe_color))
      .to_string();
    s += &style(" ".repeat((MAX_BAR_WIDTH - left)))
      .bg(Color::Color256(bg_color))
      .to_string();
  }

  let percent = ((current as f32 / total as f32 * 100.0) as u32).clamp(0, 100);
  s += &style(&format!(" {:3}% ", percent))
    .fg(Color::Color256(fe_color))
    .to_string();
  s += &style(&truncate(&text, MAX_TEXT_WIDTH)).dim().to_string();
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

  pub fn update(&self, key: &str, filename: &str, delta: u32) -> io::Result<()> {
    let mut current = self.current.lock().unwrap();
    *current = *current + delta;
    let total = *self.total.lock().unwrap();
    TERM.clear_line()?;
    let s = get_bar_str(key, total, *current, filename, GREEN, GREY);
    TERM.write_str(&s)?;
    TERM.flush()?;
    Ok(())
  }

  pub fn finish_and_clear(&self) -> io::Result<()> {
    TERM.clear_line()?;
    let total = *self.current.lock().unwrap();
    let mut s = style("finished ✔ ").green().to_string();
    s += &format!("{} files transformed. \n", total);
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

  async fn load(&self, _ctx: &BundleContext, args: &LoadArgs) -> PluginLoadHookOutput {
    let done = self.progress.done.lock().unwrap();
    if *done {
      return None;
    }
    // if matches!(_ctx.options.mode, BundleMode::Dev) {
    //   return None;
    // }
    self.progress.update("RsPack", &args.id, 1).unwrap();
    None
  }
  async fn build_end(&self, _ctx: &BundleContext) {
    let mut done = self.progress.done.lock().unwrap();
    if *done {
      return;
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
  }
}
