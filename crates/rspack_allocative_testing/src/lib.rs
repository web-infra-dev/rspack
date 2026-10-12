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
///
/// The same protection applies when the temporary borrow is inside a container.
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
/// let value = vec![Borrowed(&text)];
/// FlameGraphBuilder::with_shared_ownership().visit_root(&value);
/// ```
pub struct BorrowedReflection;

#[cfg(feature = "allocative")]
/// Owned pointers must not erase the lifetime of a static-only adapter.
///
/// ```compile_fail,E0597
/// use allocative::FlameGraphBuilder;
/// use rspack_allocative_testing::StaticOnlyBorrowed;
/// let text = String::from("temporary");
/// let value = Box::new(StaticOnlyBorrowed(&text));
/// FlameGraphBuilder::with_shared_ownership().visit_root(&value);
/// ```
///
/// ```compile_fail,E0597
/// use allocative::FlameGraphBuilder;
/// use rspack_allocative_testing::StaticOnlyBorrowed;
/// let text = String::from("temporary");
/// let value = std::sync::Arc::new(StaticOnlyBorrowed(&text));
/// FlameGraphBuilder::with_shared_ownership().visit_root(&value);
/// ```
///
/// ```compile_fail,E0597
/// use allocative::FlameGraphBuilder;
/// use rspack_allocative_testing::StaticOnlyBorrowed;
/// let text = String::from("temporary");
/// let value = std::rc::Rc::new(StaticOnlyBorrowed(&text));
/// FlameGraphBuilder::with_shared_ownership().visit_root(&value);
/// ```
pub struct StaticOnlyBorrowed<'a>(pub &'a str);

#[cfg(feature = "allocative")]
impl allocative::Allocative for StaticOnlyBorrowed<'static> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut allocative::Visitor<'b>) {
    visitor.enter_self(self).exit();
  }
}
