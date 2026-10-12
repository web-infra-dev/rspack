use rspack_allocative_driver::instrument;

#[test]
fn instrumentation_preserves_unicode_comments_and_macro_bodies() {
  let source = r#"
// 内存检查：keep this comment and its line.
mod inner { pub enum Value<T> { Empty, Data(T) } }
fn local() { enum Local { Text(String) } }
macro_rules! generated { () => { enum Generated { Text(String) } }; }
"#;
  let output = instrument(source).expect("valid source is instrumented");
  assert_eq!(output.lines().count(), source.lines().count());
  assert!(output.contains("// 内存检查：keep this comment and its line."));
  assert_eq!(
    output
      .matches("#[derive(::allocative::Allocative)]")
      .count(),
    2
  );
  assert!(output.contains("macro_rules! generated { () => { enum Generated { Text(String) } }; }"));
  assert_eq!(
    instrument(&output).expect("instrumentation is idempotent"),
    output
  );
}

#[test]
fn explicit_and_typed_adapters_are_not_duplicated() {
  let source = r#"
#[cfg_attr(allocative, derive(allocative::Allocative))]
enum Explicit { Text(String) }
enum Manual { Text(String) }
impl allocative::Allocative for Manual {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut allocative::Visitor<'b>) {}
}
enum Borrowed<'a> { Text(&'a str) }
enum Array<const N: usize> { Data([String; N]) }
"#;
  assert_eq!(
    instrument(source).expect("explicit adapters are preserved"),
    source
  );
}

#[test]
fn manual_adapters_match_qualified_paths_and_import_aliases() {
  let source = r#"
mod model {
  pub enum Direct { Text(String) }
  pub enum Parent { Text(String) }
  pub enum Aliased { Text(String) }
}
mod adapters {
  use crate::model::{Aliased as OwnedValue};
  impl allocative::Allocative for crate::model::Direct {}
  impl allocative::Allocative for super::model::Parent {}
  impl allocative::Allocative for OwnedValue {}
}
mod unrelated { pub enum Aliased { Text(String) } }
"#;
  let output = instrument(source).expect("manual adapter paths are resolved");
  assert_eq!(
    output
      .matches("#[derive(::allocative::Allocative)]")
      .count(),
    1
  );
  assert!(output.contains("mod unrelated { #[derive(::allocative::Allocative)]"));
}
#[test]
fn enums_with_const_sized_array_fields_are_instrumented() {
  let source = r#"
enum Storage<T> { Inline(SmallVec<[T; CAPACITY]>), Heap(HashSet<T>) }
enum Unevaluated { Data([String; CAPACITY]), Nested(Vec<[String; CAPACITY + 1]>) }
"#;
  let output = instrument(source).expect("const-sized arrays use typed derives");
  assert_eq!(
    output
      .matches("#[derive(::allocative::Allocative)]")
      .count(),
    2
  );
  assert_eq!(output.lines().count(), source.lines().count());
  assert_eq!(
    instrument(&output).expect("instrumentation is idempotent"),
    output
  );
}

#[test]
fn conditional_variants_are_left_for_rustc_to_evaluate() {
  let source = r#"
#[cfg(feature = "enabled")]
enum Conditional {
  #[cfg(any())]
  Removed(String),
  Present { text: String },
}
"#;
  let output = instrument(source).expect("cfg attributes remain valid Rust");
  assert!(output.contains("#[cfg(feature = \"enabled\")]"));
  assert!(output.contains("#[cfg(any())]\n  Removed(String)"));
  assert_eq!(
    output
      .matches("#[derive(::allocative::Allocative)]")
      .count(),
    1
  );
}
