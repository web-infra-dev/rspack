use std::collections::HashSet;

use swc_atoms::JsWord;

mod constant_folder;
mod renamer;

pub use constant_folder::ConstantFolder;
pub use renamer::Renamer;