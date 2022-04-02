use std::collections::HashSet;

use swc_atoms::JsWord;

mod dependency_scanner;
mod constant_folder;

pub use dependency_scanner::DependencyScanner;
pub use constant_folder::ConstantFolder;