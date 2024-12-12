mod parse;
mod repairer;
mod stringify;
mod writer;

use std::fmt::{Debug, Formatter};

pub use parse::{parse, parse_js};
pub use stringify::{print, stringify, CodegenOptions, SourceMapConfig};

pub fn gen_string_from_debug<T: Debug>(item: &T) -> String {
  let mut buf: String = String::new();
  let mut formatter = Formatter::new(&mut buf);
  // SAFTY: T is implment Debug trait
  item.fmt(&mut formatter).unwrap();
  buf
}
