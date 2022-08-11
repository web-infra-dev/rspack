use std::collections::HashMap;

use crate::{DevServerOptions, EntryItem, OutputOptions, Plugin, Resolve, Target};

#[derive(Debug)]
pub struct CompilerOptions {
  pub entry: HashMap<String, EntryItem>,
  pub context: String,
  pub dev_server: DevServerOptions,
  pub output: OutputOptions,
  pub target: Target,
  pub resolve: Resolve,
  pub plugins: Vec<Box<dyn Plugin>>,
}
