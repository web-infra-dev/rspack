use hashbrown::{HashMap, HashSet};
use rspack_symbol::Symbol;
use ustr::Ustr;

use self::visitor::TreeShakingResult;

pub mod utils;
pub mod visitor;
#[derive(Debug)]
pub struct OptimizeDependencyResult {
  pub used_symbol: HashSet<Symbol>,
  pub analyze_results: HashMap<Ustr, TreeShakingResult>,
}
