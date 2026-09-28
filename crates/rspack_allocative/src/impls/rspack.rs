//! Ownership adapters for third-party types without an `Allocative` implementation.

use std::{alloc::Layout, mem::size_of};

use smol_str::SmolStr;

use crate::{Allocative, Key, Visitor};

/// Inline/static strings own no separate allocation. Heap strings share an Arc<str>.
impl Allocative for SmolStr {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let value = self;
    let mut visitor = visitor.enter_self(value);
    if value.is_heap_allocated()
      && let Some(mut shared) = visitor.enter_shared(
        Key::new("ptr"),
        size_of::<*const str>(),
        value.as_str().as_ptr().cast(),
      )
    {
      let layout = Layout::new::<[usize; 2]>()
        .extend(Layout::for_value(value.as_str()))
        .expect("a live string must have a valid allocation layout")
        .0
        .pad_to_align();
      let mut allocation = shared.enter(Key::new("ArcInner"), layout.size());
      allocation.visit_simple(Key::new("str"), value.len());
      allocation.exit();
      shared.exit();
    }
    visitor.exit();
  }
}

impl Allocative for hstr::Atom {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    let text = self.as_ref();
    if hstr::inline_atom(text).is_none()
      && let Some(mut shared) =
        visitor.enter_shared(Key::new("ptr"), size_of::<usize>(), text.as_ptr().cast())
    {
      // hstr 4 stores dynamic atoms in ThinArc<Metadata { hash: u64 }, u8>.
      let size = Layout::new::<(usize, u64, usize)>()
        .extend(Layout::array::<u8>(text.len()).expect("a live atom must have a valid layout"))
        .expect("a live atom allocation must fit its header and text")
        .0
        .pad_to_align()
        .size();
      let mut allocation = shared.enter(Key::new("ThinArcInner"), size);
      allocation.visit_simple(Key::new("text"), text.len());
      allocation.exit();
      shared.exit();
    }
    visitor.exit();
  }
}
impl Allocative for fixedbitset::FixedBitSet {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    let mut data = visitor.enter_unique(Key::new("ptr"), size_of::<*const ()>());
    data.visit_slice(self.as_slice());
    data.exit();
    visitor.exit();
  }
}

impl<K: Allocative, V: Allocative, S> Allocative for hashlink::LinkedHashMap<K, V, S> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    visitor.visit_generic_map_fields(self);
    visitor.exit();
  }
}

impl<T: arc_swap::RefCnt + Allocative> Allocative for arc_swap::ArcSwapAny<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    let value = self.load();
    visitor.visit_field(Key::new("value"), &*value);
    visitor.exit();
  }
}
impl Allocative for json::JsonValue {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    match self {
      Self::String(value) => visitor.visit_field(Key::new("string"), value),
      Self::Array(value) => visitor.visit_field(Key::new("array"), value),
      Self::Object(value) => {
        // json::object::Object keeps its node storage private. Report that boundary,
        // while retaining visibility of recursively owned values and key text.
        visitor.visit_opaque(value);
        let mut nodes = visitor.enter_unique(Key::new("entries"), size_of::<*const ()>());
        for (key, value) in value.iter() {
          nodes.visit_simple(Key::new("key_text"), key.len());
          nodes.visit_field(Key::new("value"), value);
        }
        nodes.exit();
      }
      _ => {} // Null, booleans, numbers and short strings contain no heap allocation.
    }
    visitor.exit();
  }
}
impl<K: Allocative + Eq + std::hash::Hash, V: Allocative, S: std::hash::BuildHasher, const N: usize>
  Allocative for halfbrown::SizedHashMap<K, V, S, N>
{
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    let mut allocation = visitor.enter_unique(Key::new("ptr"), size_of::<*const ()>());
    let mut data = allocation.enter(Key::new("capacity"), self.capacity() * size_of::<(K, V)>());
    for (key, value) in self.iter() {
      data.visit_field(Key::new("key"), key);
      data.visit_field(Key::new("value"), value);
    }
    data.visit_simple(
      Key::new("unused_capacity"),
      (self.capacity() - self.len()) * size_of::<(K, V)>(),
    );
    data.exit();
    allocation.exit();
    visitor.exit();
  }
}

impl Allocative for simd_json::BorrowedValue<'_> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    match self {
      Self::Static(_) => {}
      Self::String(value) => visitor.visit_field(Key::new("string"), value),
      Self::Array(value) => visitor.visit_field(Key::new("array"), value),
      Self::Object(value) => visitor.visit_field(Key::new("object"), value),
    }
    visitor.exit();
  }
}
impl Allocative for simd_json::OwnedValue {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    match self {
      Self::Static(_) => {}
      Self::String(value) => visitor.visit_field(Key::new("string"), value),
      Self::Array(value) => visitor.visit_field(Key::new("array"), value),
      Self::Object(value) => visitor.visit_field(Key::new("object"), value),
    }
    visitor.exit();
  }
}

impl<T: Allocative, S> Allocative for hashlink::LinkedHashSet<T, S> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    visitor.visit_generic_set_fields(self);
    visitor.exit();
  }
}
impl Allocative for swc_core::common::SyntaxContext {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.enter_self(self).exit();
  }
}
impl Allocative for swc_core::common::Span {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.enter_self(self).exit();
  }
}
impl Allocative for swc_core::ecma::ast::Ident {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    visitor.visit_field(Key::new("span"), &self.span);
    visitor.visit_field(Key::new("ctxt"), &self.ctxt);
    visitor.visit_field(Key::new("sym"), &self.sym);
    visitor.exit();
  }
}
fn visit_atom_bytes<T>(value: &T, text: &[u8], visitor: &mut Visitor<'_>) {
  let mut visitor = visitor.enter_self(value);
  // hstr uses the same inline layout for UTF-8 and WTF-8 atoms. Probe the
  // public inline constructor without converting or allocating a new atom.
  let inline = text.len() <= 16 && hstr::inline_atom(&"xxxxxxxxxxxxxxxx"[..text.len()]).is_some();
  if !inline
    && let Some(mut shared) =
      visitor.enter_shared(Key::new("ptr"), size_of::<usize>(), text.as_ptr().cast())
  {
    let size = Layout::new::<(usize, u64, usize)>()
      .extend(Layout::array::<u8>(text.len()).expect("a live atom must have a valid layout"))
      .expect("a live atom allocation must fit its header and text")
      .0
      .pad_to_align()
      .size();
    let mut allocation = shared.enter(Key::new("ThinArcInner"), size);
    allocation.visit_simple(Key::new("text"), text.len());
    allocation.exit();
    shared.exit();
  }
  visitor.exit();
}
impl Allocative for swc_core::atoms::Atom {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visit_atom_bytes(self, self.as_bytes(), visitor);
  }
}
impl Allocative for swc_core::atoms::Wtf8Atom {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visit_atom_bytes(self, self.as_bytes(), visitor);
  }
}

impl<T: Allocative> Allocative for swc_config::types::BoolOrDataConfig<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    if let Some(swc_config::types::BoolOr::Data(value)) = self.inner() {
      visitor.visit_field(Key::new("data"), value);
    }
    visitor.exit();
  }
}
// These adapters name all potentially owning fields in the pinned SWC version.
// The remaining booleans/numbers/discriminants are included by enter_self.
macro_rules! owned_fields {
 ($ty:ty, $($field:ident),* $(,)?) => {
  impl Allocative for $ty {
   fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    $(visitor.visit_field(Key::new(stringify!($field)), &self.$field);)*
    visitor.exit();
   }
  }
 };
}
macro_rules! owning_variants {
 ($ty:path, $($variant:ident),* $(,)?) => {
  impl Allocative for $ty {
   fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    match self { $(Self::$variant(value) => visitor.visit_field(Key::new(stringify!($variant)), value),)* _ => {} }
    visitor.exit();
   }
  }
 };
}
owning_variants!(swc_ecma_minifier::option::terser::TerserEcmaVersion, Str);
owning_variants!(
  swc_ecma_minifier::option::terser::TerserPureGetterOption,
  Str
);
owning_variants!(
  swc_ecma_minifier::option::terser::TerserTopLevelOptions,
  Str
);
impl Allocative for swc_ecma_minifier::option::terser::TerserTopRetainOption {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    match self {
      Self::Str(value) => visitor.visit_field(Key::new("str"), value),
      Self::Seq(value) => visitor.visit_field(Key::new("seq"), value),
    }
    visitor.exit();
  }
}
owned_fields!(
  swc_ecma_minifier::option::terser::TerserCompressorOptions,
  ecma,
  global_defs,
  pure_getters,
  pure_funcs,
  top_retain,
  toplevel
);
owned_fields!(swc_ecma_minifier::option::MangleOptions, props, reserved);
owned_fields!(
  swc_ecma_minifier::js::JsMinifyFormatOptions,
  comments,
  preamble
);
impl Allocative for swc_ecma_minifier::option::ManglePropertiesOptions {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    visitor.visit_field(Key::new("reserved"), &self.reserved);
    // CachedRegex owns a private shared compiled regex; its allocation is opaque.
    if let Some(regex) = &self.regex {
      visitor.visit_opaque(regex);
    }
    visitor.exit();
  }
}
impl Allocative for swc_ecma_minifier::js::JsMinifyCommentOption {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    if let Self::PreserveRegexComments { regex } = self {
      visitor.visit_opaque(regex);
    }
    visitor.exit();
  }
}
