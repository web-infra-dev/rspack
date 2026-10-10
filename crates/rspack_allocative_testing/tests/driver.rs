use rspack_allocative_driver::{instrument, instrument_crate};

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
enum Unevaluated { Data([String; CAPACITY]) }
"#;
  assert_eq!(
    instrument(source).expect("explicit adapters are preserved"),
    source
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

#[test]
fn ownership_traits_are_instrumented_only_in_their_library() {
  let source = r#"
// 插件：preserve comments and line numbers.
#[async_trait::async_trait]
pub trait Plugin: /* existing bound */ Send + Sync { async fn apply(&self); }
pub trait Module<T> where T: Send { fn name(&self) -> &str; }
trait Dependency: ::rspack_util::allocative::Visit {}
trait Helper: Send {}
macro_rules! generated { () => { trait Plugin: Send {} }; }
"#;
  let path = "::rspack_util::allocative";
  let output = instrument_crate(source, path, "rspack_core").expect("valid traits");
  syn_check(&output);
  assert_eq!(output.lines().count(), source.lines().count());
  assert!(
    output.contains("trait Plugin: ::rspack_util::allocative::Visit + /* existing bound */ Send")
  );
  assert!(output.contains("trait Module<T>: ::rspack_util::allocative::Visit where T: Send"));
  assert!(output.contains("trait Helper: Send {}"));
  assert!(output.contains("macro_rules! generated { () => { trait Plugin: Send {} }; }"));
  assert_eq!(
    output.matches("::rspack_util::allocative::Visit").count(),
    3
  );
  assert_eq!(
    instrument_crate(&output, path, "rspack_core").unwrap(),
    output
  );
  assert_eq!(
    instrument_crate(source, path, "other_library").unwrap(),
    source
  );
  assert_eq!(instrument_crate(source, path, "ownership").unwrap(), source);
}

#[test]
fn source_trait_and_enums_share_instrumentation_without_feature_guards() {
  let source = "pub trait Source { fn size(&self) -> usize; }\nenum Owner { Text(String) }";
  let output = instrument_crate(source, "::allocative", "rspack_sources").unwrap();
  syn_check(&output);
  assert!(output.contains("trait Source: ::allocative::Visit {"));
  assert_eq!(
    output
      .matches("#[derive(::allocative::Allocative)]")
      .count(),
    1
  );
  assert!(!output.contains("feature ="));
  assert_eq!(
    instrument_crate(&output, "::allocative", "rspack_sources").unwrap(),
    output
  );
}

fn syn_check(source: &str) {
  // Re-instrumenting parses the entire transformed file, including where clauses and attributes.
  instrument(source).expect("the compiler input must remain valid Rust syntax");
}
