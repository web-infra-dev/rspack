//! Compile-time field discovery; explicit Allocative adapters take precedence.
//!
//! Automatic reflection requires a 'static type: lifetime-erased type identities
//! must not select a static-only adapter for a temporarily borrowed value.
//! References and raw pointers are leaves.

use std::{
  any::TypeId,
  mem::type_info::TypeKind,
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

// Reflection rejects impls with lifetime constraints, including `T: 'static`.
// Keep that constraint on the public Visit API and use this private bridge only
// for fields of an already-static owner. Typed specialization preserves adapters
// whose bounds cannot be rediscovered through lifetime-erased reflection.
trait FieldVisit {
  /// # Safety
  /// The concrete value must have a sized, `'static` type. Its borrow may be shorter.
  unsafe fn visit_field<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>);
}

/// Owned pointer traversal: forward visitable payloads through their actual trait
/// bound, and keep unknown static trait objects opaque.
pub(crate) trait VisitInner {
  fn visit_inner<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>);
}

// Classify pointers without requesting their layout metadata. In particular,
// the pinned compiler cannot reflect some unevaluated array-length expressions.
trait BorrowedReference {}
impl<T: ?Sized> BorrowedReference for &T {}
impl<T: ?Sized> BorrowedReference for &mut T {}
trait RawPointer {}
impl<T: ?Sized> RawPointer for *const T {}
impl<T: ?Sized> RawPointer for *mut T {}

// Avoid a layout query for arrays whose length is an unevaluated const expression.
// Our callers already know the source is sized (T, or a field of a sized T), and
// the target is a dyn trait, so use the same intrinsic without that layout query.
#[rustc_comptime]
fn vtable<T: ?Sized + Pointee<Metadata = DynMetadata<T>> + 'static>(
  id: TypeId,
) -> Option<DynMetadata<T>> {
  match std::intrinsics::reflection::type_id_vtable(id, TypeId::of::<T>()) {
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
  vtable: Option<DynMetadata<dyn FieldVisit>>,
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
  #[rustc_comptime]
  fn new<T: ?Sized>() -> Self {
    let id = std::intrinsics::reflection::type_id::<T>();
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
      TypeKind::Struct | TypeKind::Tuple => Self::fields(id, 0),
      TypeKind::Bool | TypeKind::Char | TypeKind::Int | TypeKind::Float | TypeKind::FnPtr => {
        Self::Leaf
      }
      // Enums need an active-variant accessor (derive or manual match). Unions
      // may contain uninitialized storage. Neither is guessed from layout.
      _ => Self::Opaque,
    }
  }

  #[rustc_comptime]
  fn fields(id: TypeId, variant: usize) -> Self {
    let len = id.fields(variant);
    assert!(
      len <= 256,
      "allocative reflection supports up to 256 fields per type; add an explicit adapter"
    );
    let mut fields = [EMPTY; 256];
    let mut i = 0;
    while i < len {
      let field = id.field(variant, i);
      fields[i] = FieldVisitor {
        name: field.name(),
        offset: field.offset(),
        vtable: vtable::<dyn FieldVisit>(field.type_id()),
      };
      i += 1;
    }
    Self::Fields { fields, len }
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
    let id = TypeId::of::<T>();
    let TypeKind::Enum = id.info().kind else {
      panic!("enum traversal requires an enum type");
    };
    (Key::new(id.variant(INDEX).name()), &Plan::fields(id, INDEX))
  };
  let mut visitor = visitor.enter(key, std::mem::size_of::<T>());
  if let Plan::Fields { fields, len } = plan {
    // SAFETY: T is static and the caller guarantees the active enum variant.
    unsafe { visit_fields(value, &fields[..*len], &mut visitor) };
  }
  visitor.exit();
}

// SAFETY: T must be sized and static. The fields must belong to this live value
// (and its active variant for enums).
unsafe fn visit_fields<T: ?Sized>(value: &T, fields: &[FieldVisitor], visitor: &mut Visitor<'_>) {
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
    let field_value = ptr::from_raw_parts::<dyn FieldVisit>(pointer.cast::<()>(), vtable);
    // SAFETY: Metadata belongs to this field's type, alignment was checked,
    // and the borrow does not outlive value. The enclosing static type ensures
    // its owned fields are static too. UnsafeCell has an explicit opaque
    // adapter: reflection never reads unsynchronized inner storage.
    unsafe { (&*field_value).visit_field(&mut field_visitor) };
    field_visitor.exit();
  }
}

impl<T: 'static> Visit for T {
  default fn visit_memory<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    // SAFETY: The public entry point retains the concrete type's static bound.
    unsafe { self.visit_field(visitor) };
  }
}

// Avoid even an implicit Sized bound: reflection cannot prove it for types
// whose layout depends on associated types, such as the generic Atomic<T>.
impl<T: ?Sized> FieldVisit for T {
  default unsafe fn visit_field<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
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
        // SAFETY: visit_field requires a static value, and the plan describes its fields.
        unsafe { visit_fields(self, &fields[..*len], &mut visitor) };
        visitor.exit();
      }
      Plan::Leaf => visitor.enter_self(self).exit(),
      Plan::Opaque => visitor.visit_opaque(self),
    }
  }
}

impl<T: Allocative> FieldVisit for T {
  unsafe fn visit_field<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    self.visit(visitor);
  }
}

// Forward through the actual trait bound rather than a lifetime-erased vtable.
// This also covers temporary borrows and keeps typed container adapters composable.
impl<T: Allocative> Visit for T {
  fn visit_memory<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    self.visit(visitor);
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
impl Visit for dyn Allocative + Send + '_ {
  fn visit_memory<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    self.visit(visitor);
  }
}
impl Visit for dyn Allocative + Sync + '_ {
  fn visit_memory<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    self.visit(visitor);
  }
}
impl Visit for dyn Allocative + Send + Sync + '_ {
  fn visit_memory<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    self.visit(visitor);
  }
}
