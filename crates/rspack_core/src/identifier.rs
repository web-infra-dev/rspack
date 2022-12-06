use ustr::Ustr;

pub type Identifier = Ustr;

pub trait Identifiable {
  fn identifier(&self) -> Identifier;
}
