//! Compile-time field discovery; explicit Allocative adapters take precedence.
//!
//! Automatic reflection requires a 'static type: lifetime-erased type identities
//! must not select a static-only adapter for a temporarily borrowed value.
//! References and raw pointers are leaves.

use std::{
  any::TypeId,
  mem::type_info::{Field, TypeKind},
  ptr::{self, DynMetadata, Pointee},
};

use crate::{Allocative, Key, Visitor};

/// Object-safe traversal shared by reflected values and explicit adapters.
///
/// Business structs implement this automatically. Dynamic ownership interfaces
/// use this as a supertrait so their concrete values retain reflection support.
pub trait Visit {
  fn visit_memory<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>);
}

/// Typed forwarding implemented only when the container's payload supports Visit.
/// Query the sized container instead of trying to fabricate a vtable for a DST.
pub(crate) trait VisitInner {
  fn visit_inner<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>);
}

pub(crate) fn try_visit_inner<T: 'static>(value: &T, visitor: &mut Visitor<'_>) -> bool {
  let metadata = const { vtable::<dyn VisitInner>(TypeId::of::<T>()) };
  let Some(metadata) = metadata else {
    return false;
  };
  let forwarder =
    ptr::from_raw_parts::<dyn VisitInner>(ptr::from_ref(value).cast::<()>(), metadata);
  // SAFETY: T is a static, sized container, and this vtable implements forwarding
  // for precisely T. The typed implementation borrows its payload through its API.
  unsafe { (&*forwarder).visit_inner(visitor) };
  true
}

// Classify pointers without requesting their layout metadata. In particular,
// the pinned compiler cannot reflect some unevaluated array-length expressions.
trait BorrowedReference {}
impl<T: ?Sized> BorrowedReference for &T {}
impl<T: ?Sized> BorrowedReference for &mut T {}
trait RawPointer {}
impl<T: ?Sized> RawPointer for *const T {}
impl<T: ?Sized> RawPointer for *mut T {}

// TypeId::trait_info_of calls id.info() before querying the vtable on our pinned
// nightly. That crashes for arrays whose length is an unevaluated const expression.
// Our callers already know the source is sized (T, or a field of a sized T), and
// the target is a dyn trait, so use the same intrinsic without that layout query.
const fn vtable<T: ?Sized + Pointee<Metadata = DynMetadata<T>> + 'static>(
  id: TypeId,
) -> Option<DynMetadata<T>> {
  match std::intrinsics::type_id_vtable(id, TypeId::of::<T>()) {
    Some(metadata) => {
      // SAFETY: The intrinsic returned this vtable for precisely the target T.
      // Both metadata types have the same pointer representation, as in core's
      // TypeId::trait_info_of implementation.
      Some(unsafe { std::mem::transmute::<DynMetadata<*const ()>, DynMetadata<T>>(metadata) })
    }
    None => None,
  }
}

#[derive(Clone, Copy)]
struct FieldVisitor {
  name: &'static str,
  offset: usize,
  vtable: Option<DynMetadata<dyn Visit>>,
}

const EMPTY: FieldVisitor = FieldVisitor {
  name: "",
  offset: 0,
  vtable: None,
};

// Descriptors are evaluated and borrowed as promoted constants, never stored on
// the runtime heap or copied per object. Boxing would prevent const evaluation.
#[allow(clippy::large_enum_variant)]
enum Plan {
  Adapter(DynMetadata<dyn Allocative>),
  Fields {
    fields: [FieldVisitor; 256],
    len: usize,
  },
  Leaf,
  Opaque,
}

impl Plan {
  const fn new<T: 'static>() -> Self {
    let id = TypeId::of::<T>();
    // Borrowed pointers do not imply ownership, even if their erased type has
    // an adapter for a longer lifetime.
    if vtable::<dyn BorrowedReference>(id).is_some() {
      return Self::Leaf;
    }
    if vtable::<dyn RawPointer>(id).is_some() {
      return Self::Opaque;
    }
    if let Some(adapter) = vtable::<dyn Allocative>(id) {
      return Self::Adapter(adapter);
    }
    match id.info().kind {
      TypeKind::Struct(s) => Self::fields(s.fields),
      TypeKind::Tuple(t) => Self::fields(t.fields),
      TypeKind::Bool(_)
      | TypeKind::Char(_)
      | TypeKind::Int(_)
      | TypeKind::Float(_)
      | TypeKind::FnPtr(_) => Self::Leaf,
      // Enums need an active-variant accessor (derive or manual match). Unions
      // may contain uninitialized storage. Neither is guessed from layout.
      _ => Self::Opaque,
    }
  }

  const fn fields(source: &[Field]) -> Self {
    assert!(
      source.len() <= 256,
      "allocative reflection supports up to 256 fields per type; add an explicit adapter"
    );
    let mut fields = [EMPTY; 256];
    let mut i = 0;
    while i < source.len() {
      let field = &source[i];
      fields[i] = FieldVisitor {
        name: field.name,
        offset: field.offset,
        vtable: vtable::<dyn Visit>(field.ty),
      };
      i += 1;
    }
    Self::Fields {
      fields,
      len: source.len(),
    }
  }
}

/// Visit only the fields of an enum's active variant, selected by its derive.
///
/// # Safety
/// `value` must currently contain variant `INDEX`, numbered in declaration order.
/// Visiting an inactive variant could borrow uninitialized or differently typed storage.
#[doc(hidden)]
pub unsafe fn visit_enum_variant<T: 'static, const INDEX: usize>(
  value: &T,
  visitor: &mut Visitor<'_>,
) {
  let (key, plan) = const {
    let TypeKind::Enum(info) = TypeId::of::<T>().info().kind else {
      panic!("enum traversal requires an enum type");
    };
    let variant = &info.variants[INDEX];
    (Key::new(variant.name), &Plan::fields(variant.fields))
  };
  let mut visitor = visitor.enter(key, std::mem::size_of::<T>());
  if let Plan::Fields { fields, len } = plan {
    visit_fields(value, &fields[..*len], &mut visitor);
  }
  visitor.exit();
}

// The fields must belong to this live T (and its active variant for enums).
fn visit_fields<T>(value: &T, fields: &[FieldVisitor], visitor: &mut Visitor<'_>) {
  for field in fields {
    let Some(vtable) = field.vtable else {
      visitor.report_opaque::<T>();
      continue;
    };
    // SAFETY: The compiler's offset points to an initialized field of this T.
    // Computing a raw pointer does not read packed storage.
    let pointer = unsafe { ptr::from_ref(value).cast::<u8>().add(field.offset) };
    if !(pointer as usize).is_multiple_of(vtable.align_of()) {
      visitor.report_opaque::<T>();
      continue;
    }
    let mut field_visitor = visitor.enter(Key::new(field.name), vtable.size_of());
    let field_value = ptr::from_raw_parts::<dyn Visit>(pointer.cast::<()>(), vtable);
    // SAFETY: Metadata belongs to this field's type, alignment was checked,
    // and the borrow does not outlive value. UnsafeCell has an explicit opaque
    // adapter: reflection never reads unsynchronized inner storage.
    unsafe { (&*field_value).visit_memory(&mut field_visitor) };
    field_visitor.exit();
  }
}

impl<T: 'static> Visit for T {
  fn visit_memory<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    // Borrow promoted metadata rather than copying a descriptor onto the stack
    // for every live object. Only immediate fields are stored, not the graph.
    let plan = const { &Plan::new::<T>() };
    match plan {
      Plan::Adapter(vtable) => {
        let adapter =
          ptr::from_raw_parts::<dyn Allocative>(ptr::from_ref(self).cast::<()>(), *vtable);
        // SAFETY: The compiler selected this vtable for exactly T. The pointer
        // is aligned, initialized and borrowed for the duration of this call.
        unsafe { (&*adapter).visit(visitor) };
      }
      Plan::Fields { fields, len } => {
        let mut visitor = visitor.enter_self(self);
        visit_fields(self, &fields[..*len], &mut visitor);
        visitor.exit();
      }
      Plan::Leaf => visitor.enter_self(self).exit(),
      Plan::Opaque => visitor.visit_opaque(self),
    }
  }
}

impl Visit for str {
  fn visit_memory<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    self.visit(visitor);
  }
}
impl Visit for std::ffi::OsStr {
  fn visit_memory<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    self.visit(visitor);
  }
}
#[cfg(feature = "camino")]
impl Visit for camino::Utf8Path {
  fn visit_memory<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    self.visit(visitor);
  }
}
#[cfg(feature = "triomphe")]
impl<H: Visit, T: Visit> Visit for triomphe::HeaderSlice<H, [T]> {
  fn visit_memory<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    self.visit(visitor);
  }
}
impl<T: Visit> Visit for [T] {
  fn visit_memory<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    self.visit(visitor);
  }
}
impl Visit for dyn Allocative + '_ {
  fn visit_memory<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    self.visit(visitor);
  }
}
