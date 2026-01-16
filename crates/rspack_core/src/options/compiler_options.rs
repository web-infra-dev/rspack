use crate::{
  CacheOptions, Context, Experiments, Mode, ModuleOptions, NodeOption, Optimization, OutputOptions,
  Resolve, StatsOptions,
};

#[derive(Debug)]
pub struct CompilerOptions {
  pub name: Option<String>,
  pub context: Context,
  pub output: OutputOptions,
  pub mode: Mode,
  pub resolve: Resolve,
  pub resolve_loader: Resolve,
  pub module: ModuleOptions,
  pub stats: StatsOptions,
  pub cache: CacheOptions,
  pub experiments: Experiments,
  pub node: Option<NodeOption>,
  pub optimization: Optimization,
  pub amd: Option<String>,
  pub bail: bool,
  pub __references: References,
}

pub type References = serde_json::Map<String, serde_json::Value>;
