use rspack_collections::Identifier;

#[derive(Debug)]
pub enum Event {
  StartBuild(Identifier),
  FinishDeps(Option<Identifier>),
  FinishModule(Identifier, usize),
}
