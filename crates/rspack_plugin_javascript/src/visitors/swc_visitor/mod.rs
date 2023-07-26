// mod lint;
// pub use lint::lint;

mod decorator;
pub use decorator::decorator;

mod typescript;
pub use typescript::typescript;

mod react;
pub use react::{fold_react_refresh, react};

mod define;
pub use define::define;

mod provide;
pub use provide::provide_builtin;

mod compat;
pub use compat::compat;

mod hygiene;
pub use hygiene::hygiene;
pub use swc_core::ecma::transforms::base::fixer::{fixer, paren_remover};
pub use swc_core::ecma::transforms::base::helpers::inject_helpers;
pub use swc_core::ecma::transforms::base::resolver;
pub use swc_core::ecma::transforms::compat::es2022::private_in_object;
pub use swc_core::ecma::transforms::compat::reserved_words::reserved_words;
pub use swc_core::ecma::transforms::optimization::const_modules;
pub use swc_core::ecma::transforms::optimization::json_parse;
pub use swc_core::ecma::transforms::optimization::simplifier;
pub use swc_core::ecma::transforms::optimization::simplify::dead_branch_remover;
pub use swc_core::ecma::transforms::optimization::simplify::expr_simplifier;
pub use swc_core::ecma::transforms::proposal::export_default_from;
pub use swc_core::ecma::transforms::proposal::import_assertions;
