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
use std::cell::RefCell;

use std::io;
use std::thread;
use std::time::Duration;

use console::{style, Term};

fn getBar(w: u16, done: u16, c: u8) -> String {
  let mut pixels: Vec<String> = vec![];
  let fe = format!("{}", style(" ").on_color256(c));
  let bg = format!("{}", style(" ").on_color256(0));

  for _ in 0..done {
    pixels.push(fe.clone());
  }
  for _ in 0..w - done {
    pixels.push(bg.clone());
  }
  pixels.push(String::from("\n"));
  let s = pixels.join("");
  s
}

#[derive(Debug, Clone)]
pub struct ProgressBar {
  // pub(crate) current: AtomicU32,
  // current: RefCell<u32>,
  term: Arc<Mutex<Term>>,
  current: Arc<Mutex<u32>>,
  total: u32,
  // term: Term,
}
// static TERM: Term = Term::stdout();

impl ProgressBar {
  pub fn new(total: u32) -> ProgressBar {
    // let current: AtomicU32 = AtomicU32::new(0);
    let current = Arc::new(Mutex::new(0));
    let term = Arc::new(Mutex::new(Term::stdout()));

    ProgressBar {
      total,
      current,
      term,
    }
  }

  pub fn inc(&self, delta: u32) {
    let mut x = self.current.lock().unwrap();
    *x = *x + 1;
  }

  pub fn finish_and_clear(&self) -> io::Result<()> {
    // TERM.clear_last_lines(1)?;
    Ok(())
  }
  pub fn display(&self, clear: bool) -> io::Result<()> {
    let s = getBar(self.total as u16, *self.current.lock().unwrap() as u16, 10);
    if clear {
      self.term.lock().unwrap().clear_last_lines(1)?;
    }
    self.term.lock().unwrap().write_str(&s)?;
    // println!("total: {} {}", self.total, self.current.lock().unwrap());
    // thread::sleep(Duration::from_micros(500000));
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
    progress.display(false).unwrap();
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
    self.progress.display(true).unwrap();
    None
  }
  async fn build_end(&self, _ctx: &BundleContext) {
    self.progress.finish_and_clear().unwrap();
  }
}
