use rspack_core::{ModuleGraph, ModuleIdentifier};
use rustc_hash::FxHashSet as HashSet;

pub struct ProvidedExportsPlugin<'a> {
  mg: &'a mut ModuleGraph,
}
