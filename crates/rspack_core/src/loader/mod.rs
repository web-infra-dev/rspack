mod loader_runner;

pub use loader_runner::*;
mod process_resource;
pub use process_resource::*;
use rspack_identifier::{Identifiable, Identifier};

pub struct EmptyLoader(Identifier);

impl Loader<LoaderRunnerContext> for EmptyLoader {}

impl Identifiable for EmptyLoader {
  fn identifier(&self) -> rspack_identifier::Identifier {
    self.0
  }
}
