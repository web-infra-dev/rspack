use rustc_hash::FxHashSet;
use swc_core::atoms::Atom;

#[derive(Debug)]
pub struct EnumsCollector<'a> {
  collected: &'a mut FxHashSet<Atom>,
}
