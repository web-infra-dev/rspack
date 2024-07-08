mod dropped_comments_preserver;
mod hygiene;

pub use dropped_comments_preserver::dropped_comments_preserver;
pub use swc_core::ecma::transforms::base::fixer::fixer;
pub use swc_core::ecma::transforms::base::resolver;
pub use swc_core::ecma::transforms::optimization::simplify::dead_branch_remover;
pub use swc_core::ecma::transforms::optimization::simplify::expr_simplifier;

pub use self::hygiene::hygiene;
