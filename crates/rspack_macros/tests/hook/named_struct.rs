use rspack_macros::{hook, plugin};

#[plugin]
#[derive(Debug, Clone, Hash)]
struct Plugin {
  options: String,
  state: usize,
}

mod rspack_hook {
  pub trait AsyncSeries<T> {
    fn run(&self, a: T) -> Option<()>;
    fn stage(&self) -> i32 {
      0
    }
  }
}

mod rspack_core {
  #[derive(Debug)]
  pub struct Compilation;

  pub const BASIC_STAGE: i32 = 20;
}

#[hook(rspack_hook::AsyncSeries<rspack_core::Compilation> for Plugin)]
fn process_assets(&self, a: rspack_core::Compilation) -> Option<()> {
  let this = self.inner();
  dbg!(&this.state);
  dbg!(&this.options);
  dbg!(a);
  Some(())
}

#[hook(rspack_hook::AsyncSeries<rspack_core::Compilation> for Plugin, stage = 100)]
fn make(&self, a: rspack_core::Compilation) -> Option<()> {
  let this = self.inner();
  dbg!(&this.state);
  dbg!(&this.options);
  dbg!(a);
  Some(())
}

#[hook(rspack_hook::AsyncSeries<rspack_core::Compilation> for Plugin, stage = rspack_core::BASIC_STAGE)]
fn compilation(&self, a: rspack_core::Compilation) -> Option<()> {
  let this = self.inner();
  dbg!(&this.state);
  dbg!(&this.options);
  dbg!(a);
  Some(())
}

fn main() {}
