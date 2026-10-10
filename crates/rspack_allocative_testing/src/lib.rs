//! Compile-time safety regressions for automatic reflection.

#[cfg(feature = "allocative")]
/// A temporary borrow must not use the vtable for a static-only adapter.
///
/// ```compile_fail,E0597
/// use allocative::{Allocative, FlameGraphBuilder, Visitor};
///
/// struct Borrowed<'a>(&'a str);
/// impl Allocative for Borrowed<'static> {
///     fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
///         visitor.visit_simple_sized::<Self>();
///     }
/// }
/// let text = String::from("temporary");
/// let value = Borrowed(&text);
/// FlameGraphBuilder::with_shared_ownership().visit_root(&value);
/// ```
pub struct BorrowedReflection;
