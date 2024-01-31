use std::fmt;

use rspack_error::Result;
use rustc_hash::FxHashMap as HashMap;

use crate::util::sort_push;

pub trait SyncBail<Input, Output> {
  fn run(&self, input: &mut Input) -> Result<Option<Output>>;
  fn stage(&self) -> i32 {
    0
  }
}

impl<I, O, T> SyncBail<I, O> for T
where
  T: Fn(&mut I) -> Result<Option<O>>,
{
  fn run(&self, input: &mut I) -> Result<Option<O>> {
    self(input)
  }
}

impl<I, O, T> SyncBail<I, O> for (T, i32)
where
  T: Fn(&mut I) -> Result<Option<O>>,
{
  fn run(&self, input: &mut I) -> Result<Option<O>> {
    self.0(input)
  }
  fn stage(&self) -> i32 {
    self.1
  }
}

pub struct SyncBailHook<I, O>(Vec<Box<dyn SyncBail<I, O> + Send + Sync>>);

impl<I, O> fmt::Debug for SyncBailHook<I, O> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "SyncBailHook(..)")
  }
}

impl<I, O> Default for SyncBailHook<I, O> {
  fn default() -> Self {
    Self(Vec::default())
  }
}

impl<I, O> SyncBailHook<I, O> {
  pub fn call(&self, input: &mut I) -> Result<Option<O>> {
    for hook in &self.0 {
      if let Some(res) = hook.run(input)? {
        return Ok(Some(res));
      }
    }
    Ok(None)
  }

  pub fn tap(&mut self, hook: impl SyncBail<I, O> + 'static + Send + Sync) {
    sort_push(&mut self.0, Box::new(hook), |e| e.stage());
  }
}

#[derive(Debug, Default)]
pub struct SyncBailHookMap<I, O>(HashMap<String, SyncBailHook<I, O>>);

impl<I, O> SyncBailHookMap<I, O> {
  pub fn tap(&mut self, key: String, hook: impl SyncBail<I, O> + 'static + Send + Sync) {
    self
      .0
      .entry(key)
      .or_insert_with(|| SyncBailHook::default())
      .tap(hook);
  }

  pub fn get(&self, key: &str) -> Option<&SyncBailHook<I, O>> {
    self.0.get(key)
  }
}
