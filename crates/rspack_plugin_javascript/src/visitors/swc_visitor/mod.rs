mod dropped_comments_preserver;
mod hygiene;

pub use dropped_comments_preserver::dropped_comments_preserver;
pub use swc_core::ecma::transforms::{
  base::{fixer::fixer, resolver},
  optimization::simplify::{dead_branch_remover, expr_simplifier},
};

pub use self::hygiene::hygiene;
