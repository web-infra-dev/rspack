mod lint;
pub use lint::lint;

mod decorator;
pub use decorator::decorator;

mod typescript;
pub use typescript::typescript;

mod react;
pub use react::react;

mod define;
pub use define::define;

mod compat;
pub use compat::compat;

mod build_module;
pub use build_module::build_module;

mod hygiene;
pub use hygiene::hygiene;

pub use swc_ecma_transforms::compat::es2022::private_in_object;
pub use swc_ecma_transforms::compat::reserved_words::reserved_words;
pub use swc_ecma_transforms::fixer::{fixer, paren_remover};
pub use swc_ecma_transforms::helpers::inject_helpers;
pub use swc_ecma_transforms::optimization::const_modules;
pub use swc_ecma_transforms::optimization::json_parse;
pub use swc_ecma_transforms::optimization::simplifier;
pub use swc_ecma_transforms::proposals::export_default_from;
pub use swc_ecma_transforms::proposals::import_assertions;
pub use swc_ecma_transforms::resolver;
