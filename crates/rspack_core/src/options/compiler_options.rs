use std::collections::HashMap;

use crate::{CssOptions, DevServerOptions, EntryItem, OutputOptions, Resolve, Target};

#[derive(Debug)]
pub struct CompilerOptions {
  pub entry: HashMap<String, EntryItem>,
  pub context: String,
  pub dev_server: DevServerOptions,
  pub output: OutputOptions,
  pub target: Target,
  pub resolve: Resolve,
  pub css: CssOptions,
}
