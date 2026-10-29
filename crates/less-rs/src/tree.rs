//! Arena-allocated syntax tree.
//!
//! Every node lives in the per-compilation [`bumpalo::Bump`] arena and is
//! referred to through the 16-byte, `Copy` [`Node`] enum. The structures mirror
//! less.js's `tree/*.js` classes field-for-field. Interior mutability (`Cell` /
//! `RefCell`) is used only for the fields that less.js mutates after
//! construction (visibility, lazily parsed declaration values, the rules of a
//! ruleset under evaluation, selector paths computed by the join pass, ...).

use std::cell::{Cell, OnceCell, RefCell};

use bumpalo::Bump;

pub type BVec<'a, T> = bumpalo::collections::Vec<'a, T>;

/// A position: a global byte offset over all files of the compilation.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct Pos(pub u32);

impl Pos {
  pub const NONE: Pos = Pos(u32::MAX);
  #[inline]
  pub fn is_none(self) -> bool {
    self.0 == u32::MAX
  }
  #[inline]
  pub fn opt(self) -> Option<Pos> {
    if self.is_none() { None } else { Some(self) }
  }
}

/// `!important` texts as 4-byte codes: the common spellings are fixed, the
/// rare ones (`! important`, ...) are interned (see [`rare_text`]).
#[inline]
pub fn important_code(s: &str) -> u32 {
  match s {
    "" => 0,
    "!important" => 1,
    " !important" => 2,
    _ => 3 + rare_code(s),
  }
}

#[inline]
pub fn important_text(c: u32) -> &'static str {
  match c {
    0 => "",
    1 => "!important",
    2 => " !important",
    _ => rare_text(c - 3),
  }
}

/// Process-wide interner for the rare texts behind 4-byte codes (unusual
/// `!important` spellings and combinators): they are matches of narrow
/// patterns, so few distinct ones exist.
#[cold]
fn rare_code(s: &str) -> u32 {
  let mut t = rare_texts().lock().unwrap_or_else(|e| e.into_inner());
  if let Some(&i) = t.1.get(s) {
    return i;
  }
  let text: &'static str = Box::leak(s.to_string().into_boxed_str());
  let i = t.0.len() as u32;
  t.0.push(text);
  t.1.insert(text, i);
  i
}

#[cold]
pub fn rare_text(c: u32) -> &'static str {
  rare_texts().lock().unwrap_or_else(|e| e.into_inner()).0[c as usize]
}

type RareTexts = (Vec<&'static str>, rustc_hash::FxHashMap<&'static str, u32>);

/// Texts by code, and codes by text (each distinct text is kept once).
fn rare_texts() -> &'static std::sync::Mutex<RareTexts> {
  static RARE_TEXTS: std::sync::OnceLock<std::sync::Mutex<RareTexts>> = std::sync::OnceLock::new();
  RARE_TEXTS.get_or_init(Default::default)
}

/// `visibilityBlocks` / `nodeVisible` from `tree/node.js`, packed in one word:
/// the block count times 4 plus 0 (unset), 1 (invisible) or 2 (visible).
#[derive(Default, Debug)]
pub struct Vis(Cell<i32>);

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct VisInfo {
  pub blocks: i32,
  pub visible: Option<bool>,
}

impl Vis {
  fn pack(blocks: i32, visible: Option<bool>) -> i32 {
    blocks.wrapping_mul(4)
      | match visible {
        None => 0,
        Some(false) => 1,
        Some(true) => 2,
      }
  }
  fn blocks(&self) -> i32 {
    self.0.get() >> 2
  }
  fn visible(&self) -> Option<bool> {
    match self.0.get() & 3 {
      0 => None,
      1 => Some(false),
      _ => Some(true),
    }
  }
  pub fn new(info: VisInfo) -> Vis {
    Vis(Cell::new(Vis::pack(info.blocks, info.visible)))
  }
  #[inline]
  pub fn blocks_visibility(&self) -> bool {
    self.blocks() != 0
  }
  pub fn add_block(&self) {
    self.0.set(self.0.get().wrapping_add(4));
  }
  pub fn remove_block(&self) {
    self.0.set(self.0.get().wrapping_sub(4));
  }
  pub fn ensure_visibility(&self) {
    self.set_visible(Some(true));
  }
  pub fn ensure_invisibility(&self) {
    self.set_visible(Some(false));
  }
  #[inline]
  pub fn is_visible(&self) -> bool {
    self.0.get() & 3 == 2
  }
  pub fn info(&self) -> VisInfo {
    VisInfo {
      blocks: self.blocks(),
      visible: self.visible(),
    }
  }
  pub fn copy_from(&self, info: VisInfo) {
    self.0.set(Vis::pack(info.blocks, info.visible));
  }
  pub fn set_visible(&self, v: Option<bool>) {
    self.0.set(Vis::pack(self.blocks(), v));
  }
}

#[derive(Copy, Clone, Debug)]
pub struct DebugInfo<'a> {
  pub line_number: u32,
  pub file_name: &'a str,
}

/// The polymorphic node handle.
#[derive(Copy, Clone)]
pub enum Node<'a> {
  Anonymous(&'a Anonymous<'a>),
  Assignment(&'a Assignment<'a>),
  AtRule(&'a AtRule<'a>),
  Media(&'a AtRule<'a>),
  Container(&'a AtRule<'a>),
  Attribute(&'a Attribute<'a>),
  Call(&'a Call<'a>),
  Color(&'a Color<'a>),
  Comment(&'a Comment<'a>),
  Condition(&'a Condition<'a>),
  Declaration(&'a Declaration<'a>),
  DetachedRuleset(&'a DetachedRuleset<'a>),
  Dimension(&'a Dimension<'a>),
  Element(&'a Element<'a>),
  Expression(&'a Expression<'a>),
  Extend(&'a Extend<'a>),
  Import(&'a Import<'a>),
  JavaScript(&'a JavaScript<'a>),
  Keyword(&'a Keyword<'a>),
  MixinCall(&'a MixinCall<'a>),
  MixinDefinition(&'a Ruleset<'a>),
  NamespaceValue(&'a NamespaceValue<'a>),
  Negative(&'a Negative<'a>),
  Operation(&'a Operation<'a>),
  Paren(&'a Paren<'a>),
  Property(&'a Property<'a>),
  QueryInParens(&'a QueryInParens<'a>),
  Quoted(&'a Quoted<'a>),
  Ruleset(&'a Ruleset<'a>),
  Selector(&'a Selector<'a>),
  UnicodeDescriptor(&'a UnicodeDescriptor<'a>),
  Url(&'a Url<'a>),
  Value(&'a Value<'a>),
  Variable(&'a Variable<'a>),
  VariableCall(&'a VariableCall<'a>),
  /// A JS array of rules used as a value (e.g. the result of a mixin call
  /// stored in a variable).
  Rules(&'a RuleList<'a>),
}

impl<'a> Node<'a> {
  pub fn type_name(&self) -> &'static str {
    match self {
      Node::Anonymous(_) => "Anonymous",
      Node::Assignment(_) => "Assignment",
      Node::AtRule(_) => "AtRule",
      Node::Media(_) => "Media",
      Node::Container(_) => "Container",
      Node::Attribute(_) => "Attribute",
      Node::Call(_) => "Call",
      Node::Color(_) => "Color",
      Node::Comment(_) => "Comment",
      Node::Condition(_) => "Condition",
      Node::Declaration(_) => "Declaration",
      Node::DetachedRuleset(_) => "DetachedRuleset",
      Node::Dimension(_) => "Dimension",
      Node::Element(_) => "Element",
      Node::Expression(_) => "Expression",
      Node::Extend(_) => "Extend",
      Node::Import(_) => "Import",
      Node::JavaScript(_) => "JavaScript",
      Node::Keyword(_) => "Keyword",
      Node::MixinCall(_) => "MixinCall",
      Node::MixinDefinition(_) => "MixinDefinition",
      Node::NamespaceValue(_) => "NamespaceValue",
      Node::Negative(_) => "Negative",
      Node::Operation(_) => "Operation",
      Node::Paren(_) => "Paren",
      Node::Property(_) => "Property",
      Node::QueryInParens(_) => "QueryInParens",
      Node::Quoted(_) => "Quoted",
      Node::Ruleset(_) => "Ruleset",
      Node::Selector(_) => "Selector",
      Node::UnicodeDescriptor(_) => "UnicodeDescriptor",
      Node::Url(_) => "Url",
      Node::Value(_) => "Value",
      Node::Variable(_) => "Variable",
      Node::VariableCall(_) => "VariableCall",
      Node::Rules(_) => "Array",
    }
  }

  /// Visibility bookkeeping, for the node types that carry it.
  pub fn vis(&self) -> Option<&'a Vis> {
    Some(match *self {
      Node::Anonymous(n) => &n.vis,
      Node::AtRule(n) | Node::Media(n) | Node::Container(n) => &n.vis,
      Node::Comment(n) => &n.vis,
      Node::Declaration(n) => &n.vis,
      Node::Element(n) => &n.vis,
      Node::Extend(n) => &n.vis,
      Node::Import(n) => &n.vis,
      Node::MixinCall(n) => &n.vis,
      Node::MixinDefinition(n) | Node::Ruleset(n) => &n.vis,
      Node::Selector(n) => &n.vis,
      Node::Call(n) => &n.vis,
      Node::Keyword(n) => &n.vis,
      Node::Quoted(n) => &n.vis,
      Node::Value(n) => &n.vis,
      Node::Expression(n) => &n.vis,
      Node::DetachedRuleset(n) => &n.vis,
      Node::VariableCall(n) => &n.vis,
      _ => return None,
    })
  }

  pub fn add_visibility_block(&self) {
    if let Some(v) = self.vis() {
      v.add_block();
    }
  }
  pub fn is_visible(&self) -> bool {
    self.vis().map(|v| v.is_visible()).unwrap_or(false)
  }
  pub fn visibility_info(&self) -> VisInfo {
    self.vis().map(|v| v.info()).unwrap_or_default()
  }
  pub fn copy_visibility_info(&self, info: VisInfo) {
    if let Some(v) = self.vis() {
      v.copy_from(info);
    }
  }

  /// `allowRoot` from less.js.
  pub fn allow_root(&self) -> bool {
    match self {
      Node::Anonymous(_)
      | Node::AtRule(_)
      | Node::Media(_)
      | Node::Container(_)
      | Node::Comment(_)
      | Node::Declaration(_)
      | Node::Extend(_)
      | Node::Import(_)
      | Node::MixinCall(_)
      | Node::MixinDefinition(_)
      | Node::Ruleset(_)
      | Node::VariableCall(_) => true,
      Node::Quoted(q) => q.escaped,
      _ => false,
    }
  }

  pub fn is_ruleset_like(&self) -> bool {
    match self {
      Node::Ruleset(_) | Node::MixinDefinition(_) | Node::Media(_) | Node::Container(_) => true,
      Node::AtRule(a) => a.rules.get().is_some() || !a.is_charset(),
      Node::Anonymous(a) => a.ruleset_like,
      _ => false,
    }
  }

  /// `node.isRuleset` (rulesets and mixin definitions).
  pub fn as_ruleset(&self) -> Option<&'a Ruleset<'a>> {
    match *self {
      Node::Ruleset(r) | Node::MixinDefinition(r) => Some(r),
      _ => None,
    }
  }

  pub fn ptr_eq(&self, other: &Node<'a>) -> bool {
    fn addr<T>(r: &T) -> usize {
      r as *const T as usize
    }
    let a = self.addr();
    let b = other.addr();
    a == b && std::mem::discriminant(self) == std::mem::discriminant(other) && a != 0 && {
      let _ = addr::<u8>;
      true
    }
  }

  pub fn addr(&self) -> usize {
    match *self {
      Node::Anonymous(n) => n as *const _ as usize,
      Node::Assignment(n) => n as *const _ as usize,
      Node::AtRule(n) | Node::Media(n) | Node::Container(n) => n as *const _ as usize,
      Node::Attribute(n) => n as *const _ as usize,
      Node::Call(n) => n as *const _ as usize,
      Node::Color(n) => n as *const _ as usize,
      Node::Comment(n) => n as *const _ as usize,
      Node::Condition(n) => n as *const _ as usize,
      Node::Declaration(n) => n as *const _ as usize,
      Node::DetachedRuleset(n) => n as *const _ as usize,
      Node::Dimension(n) => n as *const _ as usize,
      Node::Element(n) => n as *const _ as usize,
      Node::Expression(n) => n as *const _ as usize,
      Node::Extend(n) => n as *const _ as usize,
      Node::Import(n) => n as *const _ as usize,
      Node::JavaScript(n) => n as *const _ as usize,
      Node::Keyword(n) => n as *const _ as usize,
      Node::MixinCall(n) => n as *const _ as usize,
      Node::MixinDefinition(n) | Node::Ruleset(n) => n as *const _ as usize,
      Node::NamespaceValue(n) => n as *const _ as usize,
      Node::Negative(n) => n as *const _ as usize,
      Node::Operation(n) => n as *const _ as usize,
      Node::Paren(n) => n as *const _ as usize,
      Node::Property(n) => n as *const _ as usize,
      Node::QueryInParens(n) => n as *const _ as usize,
      Node::Quoted(n) => n as *const _ as usize,
      Node::Selector(n) => n as *const _ as usize,
      Node::UnicodeDescriptor(n) => n as *const _ as usize,
      Node::Url(n) => n as *const _ as usize,
      Node::Value(n) => n as *const _ as usize,
      Node::Variable(n) => n as *const _ as usize,
      Node::VariableCall(n) => n as *const _ as usize,
      Node::Rules(n) => n as *const _ as usize,
    }
  }

  /// `getIndex()` (as a global position).
  pub fn pos(&self) -> Pos {
    match *self {
      Node::Anonymous(n) => n.pos,
      Node::AtRule(n) | Node::Media(n) | Node::Container(n) => n.pos,
      Node::Call(n) => n.pos,
      Node::Comment(n) => n.pos,
      Node::Condition(n) => n.pos,
      Node::Declaration(n) => n.pos,
      Node::Element(n) => n.pos,
      Node::Extend(n) => n.pos,
      Node::Import(n) => n.pos,
      Node::JavaScript(n) => n.pos,
      Node::MixinCall(n) => n.pos,
      Node::NamespaceValue(n) => n.pos,
      Node::Property(n) => n.pos,
      Node::QueryInParens(n) => n.pos,
      Node::Quoted(n) => n.pos,
      Node::Selector(n) => n.pos,
      Node::Url(n) => n.pos,
      Node::Variable(n) => n.pos,
      Node::VariableCall(n) => n.pos,
      Node::Dimension(n) => n.pos.get(),
      Node::Color(n) => n.pos.get(),
      Node::Keyword(n) => n.pos.get(),
      Node::Expression(n) => n.pos,
      Node::Value(n) => n.pos,
      Node::Paren(n) => n.value.pos(),
      Node::Negative(n) => n.value.pos(),
      Node::Operation(n) => n.operands[0].pos(),
      _ => Pos::NONE,
    }
  }

  /// `noSpacing` (only meaningful on a few node types).
  pub fn no_spacing(&self) -> Option<bool> {
    match self {
      Node::Expression(e) => e.no_spacing.get(),
      Node::Paren(p) => p.no_spacing.get(),
      Node::Keyword(k) => k.no_spacing.get(),
      Node::Variable(v) => v.no_spacing.get(),
      Node::Anonymous(a) => a.no_spacing.get(),
      _ => None,
    }
  }
  pub fn set_no_spacing(&self, v: Option<bool>) {
    match self {
      Node::Expression(e) => e.no_spacing.set(v),
      Node::Paren(p) => p.no_spacing.set(v),
      Node::Keyword(k) => k.no_spacing.set(v),
      Node::Variable(x) => x.no_spacing.set(v),
      Node::Anonymous(a) => a.no_spacing.set(v),
      _ => {}
    }
  }

  pub fn is_comment(&self) -> bool {
    matches!(self, Node::Comment(_))
  }

  /// JS `node.value` when it is a string (Keyword, Anonymous, Quoted, ...).
  pub fn str_value(&self) -> Option<&'a str> {
    match *self {
      Node::Keyword(k) => Some(k.value),
      Node::Anonymous(a) => Some(a.value),
      Node::Quoted(q) => Some(q.value),
      Node::UnicodeDescriptor(u) => Some(u.value),
      Node::Color(c) => c.value.get(),
      _ => None,
    }
  }

  /// JS `Array.isArray(node.value)` → the array.
  pub fn array_value(&self) -> Option<&'a [Node<'a>]> {
    match *self {
      Node::Expression(e) => Some(e.value),
      Node::Value(v) => Some(v.value),
      Node::Rules(r) => Some(r.items),
      _ => None,
    }
  }
}

/// A list of nodes behind a thin pointer (keeps `Node` at 16 bytes).
pub struct RuleList<'a> {
  pub items: &'a [Node<'a>],
}

impl<'a> std::ops::Deref for RuleList<'a> {
  type Target = [Node<'a>];
  fn deref(&self) -> &[Node<'a>] {
    self.items
  }
}

// ---------------------------------------------------------------------------
// Leaf and value nodes
// ---------------------------------------------------------------------------

pub struct Anonymous<'a> {
  pub vis: Vis,
  pub value: &'a str,
  pub pos: Pos,
  pub map_lines: bool,
  pub ruleset_like: bool,
  pub prevent_reparse: bool,
  pub no_spacing: Cell<Option<bool>>,
  /// Set when the value was `null`/`undefined` in less.js.
  pub is_null: bool,
  /// Whether less.js gave this node a `fileInfo` (only then is it source
  /// mapped).
  pub has_file: bool,
}

pub struct Assignment<'a> {
  pub key: &'a str,
  pub value: Node<'a>,
}

pub struct Attribute<'a> {
  pub key: StrOrNode<'a>,
  pub op: Option<&'a str>,
  pub value: Option<StrOrNode<'a>>,
  pub cif: Option<&'a str>,
}

#[derive(Copy, Clone)]
pub enum StrOrNode<'a> {
  Str(&'a str),
  Node(Node<'a>),
}

pub struct Call<'a> {
  pub vis: Vis,
  pub name: &'a str,
  pub args: &'a [Node<'a>],
  pub pos: Pos,
  pub calc: bool,
}

pub struct Color<'a> {
  pub rgb: [f64; 3],
  pub alpha: f64,
  /// The original form (keyword, `#abc`, `rgb`, `hsl`, ...).
  pub value: Cell<Option<&'a str>>,
  pub pos: Cell<Pos>,
}

pub struct Comment<'a> {
  pub vis: Vis,
  pub value: &'a str,
  pub is_line_comment: bool,
  pub pos: Pos,
  pub debug_info: Cell<Option<DebugInfo<'a>>>,
}

impl<'a> Comment<'a> {
  pub fn is_silent(&self, compress: bool) -> bool {
    let is_compressed = compress && self.value.as_bytes().get(2) != Some(&b'!');
    self.is_line_comment || is_compressed
  }
}

pub struct Condition<'a> {
  pub op: &'a str,
  pub lvalue: Node<'a>,
  pub rvalue: Node<'a>,
  pub pos: Pos,
  pub negate: Cell<bool>,
}

pub struct Dimension<'a> {
  pub value: f64,
  pub unit: &'a Unit<'a>,
  pub pos: Cell<Pos>,
}

pub struct Unit<'a> {
  pub numerator: &'a [&'a str],
  pub denominator: &'a [&'a str],
  pub backup_unit: Option<&'a str>,
}

pub static EMPTY_UNIT: Unit<'static> = Unit {
  numerator: &[],
  denominator: &[],
  backup_unit: None,
};

/// Shared instances of the common single units (`px` with backup `px`, ...);
/// units are immutable.
fn common_unit(
  numerator: &[&str],
  denominator: &[&str],
  backup: Option<&str>,
) -> Option<&'static Unit<'static>> {
  if numerator.len() != 1 || !denominator.is_empty() || backup != Some(numerator[0]) {
    return None;
  }
  Some(match numerator[0] {
    "px" => {
      static U: Unit<'static> = Unit {
        numerator: &["px"],
        denominator: &[],
        backup_unit: Some("px"),
      };
      &U
    }
    "em" => {
      static U: Unit<'static> = Unit {
        numerator: &["em"],
        denominator: &[],
        backup_unit: Some("em"),
      };
      &U
    }
    "rem" => {
      static U: Unit<'static> = Unit {
        numerator: &["rem"],
        denominator: &[],
        backup_unit: Some("rem"),
      };
      &U
    }
    "%" => {
      static U: Unit<'static> = Unit {
        numerator: &["%"],
        denominator: &[],
        backup_unit: Some("%"),
      };
      &U
    }
    "ex" => {
      static U: Unit<'static> = Unit {
        numerator: &["ex"],
        denominator: &[],
        backup_unit: Some("ex"),
      };
      &U
    }
    "ch" => {
      static U: Unit<'static> = Unit {
        numerator: &["ch"],
        denominator: &[],
        backup_unit: Some("ch"),
      };
      &U
    }
    "vw" => {
      static U: Unit<'static> = Unit {
        numerator: &["vw"],
        denominator: &[],
        backup_unit: Some("vw"),
      };
      &U
    }
    "vh" => {
      static U: Unit<'static> = Unit {
        numerator: &["vh"],
        denominator: &[],
        backup_unit: Some("vh"),
      };
      &U
    }
    "vmin" => {
      static U: Unit<'static> = Unit {
        numerator: &["vmin"],
        denominator: &[],
        backup_unit: Some("vmin"),
      };
      &U
    }
    "vmax" => {
      static U: Unit<'static> = Unit {
        numerator: &["vmax"],
        denominator: &[],
        backup_unit: Some("vmax"),
      };
      &U
    }
    "cm" => {
      static U: Unit<'static> = Unit {
        numerator: &["cm"],
        denominator: &[],
        backup_unit: Some("cm"),
      };
      &U
    }
    "mm" => {
      static U: Unit<'static> = Unit {
        numerator: &["mm"],
        denominator: &[],
        backup_unit: Some("mm"),
      };
      &U
    }
    "in" => {
      static U: Unit<'static> = Unit {
        numerator: &["in"],
        denominator: &[],
        backup_unit: Some("in"),
      };
      &U
    }
    "pt" => {
      static U: Unit<'static> = Unit {
        numerator: &["pt"],
        denominator: &[],
        backup_unit: Some("pt"),
      };
      &U
    }
    "pc" => {
      static U: Unit<'static> = Unit {
        numerator: &["pc"],
        denominator: &[],
        backup_unit: Some("pc"),
      };
      &U
    }
    "q" => {
      static U: Unit<'static> = Unit {
        numerator: &["q"],
        denominator: &[],
        backup_unit: Some("q"),
      };
      &U
    }
    "s" => {
      static U: Unit<'static> = Unit {
        numerator: &["s"],
        denominator: &[],
        backup_unit: Some("s"),
      };
      &U
    }
    "ms" => {
      static U: Unit<'static> = Unit {
        numerator: &["ms"],
        denominator: &[],
        backup_unit: Some("ms"),
      };
      &U
    }
    "deg" => {
      static U: Unit<'static> = Unit {
        numerator: &["deg"],
        denominator: &[],
        backup_unit: Some("deg"),
      };
      &U
    }
    "rad" => {
      static U: Unit<'static> = Unit {
        numerator: &["rad"],
        denominator: &[],
        backup_unit: Some("rad"),
      };
      &U
    }
    "grad" => {
      static U: Unit<'static> = Unit {
        numerator: &["grad"],
        denominator: &[],
        backup_unit: Some("grad"),
      };
      &U
    }
    "turn" => {
      static U: Unit<'static> = Unit {
        numerator: &["turn"],
        denominator: &[],
        backup_unit: Some("turn"),
      };
      &U
    }
    "dpi" => {
      static U: Unit<'static> = Unit {
        numerator: &["dpi"],
        denominator: &[],
        backup_unit: Some("dpi"),
      };
      &U
    }
    "dpcm" => {
      static U: Unit<'static> = Unit {
        numerator: &["dpcm"],
        denominator: &[],
        backup_unit: Some("dpcm"),
      };
      &U
    }
    "dppx" => {
      static U: Unit<'static> = Unit {
        numerator: &["dppx"],
        denominator: &[],
        backup_unit: Some("dppx"),
      };
      &U
    }
    "fr" => {
      static U: Unit<'static> = Unit {
        numerator: &["fr"],
        denominator: &[],
        backup_unit: Some("fr"),
      };
      &U
    }
    "x" => {
      static U: Unit<'static> = Unit {
        numerator: &["x"],
        denominator: &[],
        backup_unit: Some("x"),
      };
      &U
    }
    _ => return None,
  })
}

pub struct Expression<'a> {
  pub vis: Vis,
  pub value: &'a [Node<'a>],
  pub no_spacing: Cell<Option<bool>>,
  pub parens: bool,
  pub parens_in_op: Cell<bool>,
  pub pos: Pos,
}

pub struct JavaScript<'a> {
  pub expression: &'a str,
  pub escaped: bool,
  pub pos: Pos,
}

pub struct Keyword<'a> {
  pub vis: Vis,
  pub value: &'a str,
  pub no_spacing: Cell<Option<bool>>,
  pub pos: Cell<Pos>,
}

pub struct Negative<'a> {
  pub value: Node<'a>,
}

pub struct Operation<'a> {
  pub op: &'a str,
  pub operands: [Node<'a>; 2],
  pub is_spaced: bool,
}

pub struct Paren<'a> {
  pub value: Node<'a>,
  pub no_spacing: Cell<Option<bool>>,
}

pub struct Property<'a> {
  pub name: &'a str,
  pub pos: Pos,
  pub evaluating: Cell<bool>,
}

pub struct QueryInParens<'a> {
  pub op: &'a str,
  pub lvalue: Node<'a>,
  pub mvalue: Node<'a>,
  pub op2: Option<&'a str>,
  pub rvalue: Option<Node<'a>>,
  pub pos: Pos,
}

pub struct Quoted<'a> {
  pub vis: Vis,
  pub quote: &'a str,
  pub value: &'a str,
  pub escaped: bool,
  pub pos: Pos,
  /// Values produced by `permissiveValue` also interpolate bare `@var` and
  /// `$prop` (the custom `variableRegex`/`propRegex` in less.js).
  pub permissive: bool,
}

pub struct UnicodeDescriptor<'a> {
  pub value: &'a str,
}

pub struct Url<'a> {
  pub value: Node<'a>,
  pub pos: Pos,
  pub is_evald: bool,
}

pub struct Value<'a> {
  pub vis: Vis,
  pub value: &'a [Node<'a>],
  pub pos: Pos,
}

pub struct Variable<'a> {
  pub name: &'a str,
  pub pos: Pos,
  pub evaluating: Cell<bool>,
  pub no_spacing: Cell<Option<bool>>,
}

pub struct VariableCall<'a> {
  pub vis: Vis,
  pub variable: &'a str,
  pub pos: Pos,
}

pub struct NamespaceValue<'a> {
  pub value: Node<'a>,
  pub lookups: &'a [&'a str],
  pub pos: Pos,
}

pub struct DetachedRuleset<'a> {
  pub vis: Vis,
  pub ruleset: Node<'a>,
  pub frames: Option<&'a [&'a Ruleset<'a>]>,
}

// ---------------------------------------------------------------------------
// Selectors
// ---------------------------------------------------------------------------

/// A combinator's text as a 4-byte code: the built-in ones are fixed, others
/// (`/deep/`, ...) are interned for the process (see [`rare_text`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Combinator<'a> {
  c: u32,
  _a: std::marker::PhantomData<&'a str>,
}

const COMBINATORS: [&str; 8] = ["", " ", ">", "+", "~", "|", "^", "^^"];

impl<'a> Combinator<'a> {
  const fn code(c: u32) -> Combinator<'static> {
    Combinator {
      c,
      _a: std::marker::PhantomData,
    }
  }
  /// `new Combinator(value)` for the built-in combinators.
  pub fn new(value: &str) -> Combinator<'static> {
    match value.trim() {
      "" if value == " " => Combinator::SPACE,
      v => match COMBINATORS.iter().position(|c| *c == v) {
        Some(i) => Combinator::code(i as u32),
        None => Combinator::EMPTY,
      },
    }
  }
  /// `new Combinator(value)` for any value (e.g. `/deep/`).
  pub fn new_in(_a: Alloc<'a>, value: &'a str) -> Combinator<'a> {
    let c = Combinator::new(value);
    if c.value() == value.trim() || (value == " " && c.value() == " ") {
      return c;
    }
    Combinator::code(COMBINATORS.len() as u32 + rare_code(value.trim()))
  }
  #[inline]
  pub fn value(&self) -> &'a str {
    match COMBINATORS.get(self.c as usize) {
      Some(v) => v,
      None => rare_text(self.c - COMBINATORS.len() as u32),
    }
  }
  pub fn empty_or_whitespace(&self) -> bool {
    self.c <= 1
  }
  pub const EMPTY: Combinator<'static> = Combinator::code(0);
  pub const SPACE: Combinator<'static> = Combinator::code(1);
}

#[derive(Copy, Clone)]
pub enum ElValue<'a> {
  Str(&'a str),
  Node(Node<'a>),
}

impl<'a> ElValue<'a> {
  pub fn as_str(&self) -> Option<&'a str> {
    match *self {
      ElValue::Str(s) => Some(s),
      _ => None,
    }
  }
}

pub struct Element<'a> {
  pub vis: Vis,
  pub combinator: Cell<Combinator<'a>>,
  pub value: ElValue<'a>,
  pub is_variable: bool,
  pub pos: Pos,
}

pub struct Selector<'a> {
  pub vis: Vis,
  pub elements: &'a [&'a Element<'a>],
  pub evald_condition: Cell<bool>,
  pub media_empty: Cell<bool>,
  pub pos: Pos,
  /// The rarely present parts, out of line (never shared between selectors).
  extra: Cell<Option<&'a SelExtra<'a>>>,
}

/// A selector's `list` / `condition`, `extendList` and `mixinElements()`
/// cache: most selectors have none of them.
pub struct SelExtra<'a> {
  aux: Option<SelAux<'a>>,
  extend_list: Cell<Option<&'a [&'a Extend<'a>]>>,
  mixin_elements: OnceCell<&'a [&'a str]>,
}

pub enum SelAux<'a> {
  /// The `(a, b)` paren list form: `new Selector([Selector, Anonymous(','), Selector])`.
  List(&'a [Node<'a>]),
  Condition(Node<'a>),
}

impl<'a> Selector<'a> {
  /// `new Selector(elements, extendList, condition/list, ...)`.
  pub fn new(
    a: Alloc<'a>,
    elements: &'a [&'a Element<'a>],
    aux: Option<SelAux<'a>>,
    extend_list: Option<&'a [&'a Extend<'a>]>,
    pos: Pos,
    vis: VisInfo,
  ) -> Selector<'a> {
    let evald_condition = !matches!(aux, Some(SelAux::Condition(_)));
    let extra = (aux.is_some() || extend_list.is_some()).then(|| {
      a.alloc(SelExtra {
        aux,
        extend_list: Cell::new(extend_list),
        mixin_elements: OnceCell::new(),
      })
    });
    Selector {
      vis: Vis::new(vis),
      elements,
      evald_condition: Cell::new(evald_condition),
      media_empty: Cell::new(false),
      pos,
      extra: Cell::new(extra),
    }
  }
  fn extra_in(&self, a: Alloc<'a>) -> &'a SelExtra<'a> {
    match self.extra.get() {
      Some(x) => x,
      None => {
        let x = a.alloc(SelExtra {
          aux: None,
          extend_list: Cell::new(None),
          mixin_elements: OnceCell::new(),
        });
        self.extra.set(Some(x));
        x
      }
    }
  }
  pub fn has_aux(&self) -> bool {
    self.extra.get().is_some_and(|x| x.aux.is_some())
  }
  pub fn list(&self) -> Option<&'a [Node<'a>]> {
    match self.extra.get().and_then(|x| x.aux.as_ref()) {
      Some(SelAux::List(l)) => Some(l),
      _ => None,
    }
  }
  pub fn condition(&self) -> Option<Node<'a>> {
    match self.extra.get().and_then(|x| x.aux.as_ref()) {
      Some(SelAux::Condition(c)) => Some(*c),
      _ => None,
    }
  }
  pub fn extend_list(&self) -> Option<&'a [&'a Extend<'a>]> {
    self.extra.get().and_then(|x| x.extend_list.get())
  }
  pub fn set_extend_list(&self, a: Alloc<'a>, v: Option<&'a [&'a Extend<'a>]>) {
    if v.is_some() || self.extra.get().is_some() {
      self.extra_in(a).extend_list.set(v);
    }
  }
  /// The `mixinElements()` cache.
  pub fn mixin_elements_cache(&self, a: Alloc<'a>) -> &'a OnceCell<&'a [&'a str]> {
    &self.extra_in(a).mixin_elements
  }
  pub fn mixin_elements_cached(&self) -> Option<&'a [&'a str]> {
    self
      .extra
      .get()
      .and_then(|x| x.mixin_elements.get().copied())
  }
  pub fn is_just_parent_selector(&self) -> bool {
    !self.media_empty.get()
      && self.elements.len() == 1
      && matches!(self.elements[0].value, ElValue::Str("&"))
      && {
        let c = self.elements[0].combinator.get().value();
        c == " " || c.is_empty()
      }
  }
}

pub struct Extend<'a> {
  pub vis: Vis,
  pub selector: &'a Selector<'a>,
  pub option: Option<&'a str>,
  pub pos: Pos,
  pub object_id: u32,
  pub parent_ids: RefCell<BVec<'a, u32>>,
  pub allow_before: bool,
  pub allow_after: bool,
  pub self_selectors: Cell<&'a [&'a Selector<'a>]>,
  pub ruleset: Cell<Option<&'a Ruleset<'a>>>,
  pub first_extend_on_this_selector_path: Cell<bool>,
  pub has_found_matches: Cell<bool>,
}

// ---------------------------------------------------------------------------
// Rules
// ---------------------------------------------------------------------------

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Merge {
  None,
  Comma,
  Space,
}

#[derive(Copy, Clone)]
pub enum DeclName<'a> {
  Str(&'a str),
  Parts(&'a [Node<'a>]),
}

pub struct Declaration<'a> {
  pub vis: Vis,
  /// Shared with the declaration it was evaluated from when unchanged.
  pub name: &'a DeclName<'a>,
  pub value: Cell<Node<'a>>,
  /// The `!important` text as an [`important_code`].
  pub important: Cell<u32>,
  pub merge: Cell<Merge>,
  pub pos: Pos,
  pub inline: bool,
  pub variable: bool,
  /// [`Declaration::PARSED`] | [`Declaration::SHARED`]
  pub flags: Cell<u8>,
}

impl<'a> Declaration<'a> {
  /// `parsed`: the value was reparsed by a lookup (`parseValue`).
  pub const PARSED: u8 = 1;
  /// Evaluation returned this declaration itself (its evaluation would
  /// copy it unchanged), so it can occur at several places of the tree;
  /// per-occurrence changes go to copies.
  pub const SHARED: u8 = 2;

  pub fn important(&self) -> &'static str {
    important_text(self.important.get())
  }
  pub fn set_important(&self, s: &str) {
    self.important.set(important_code(s));
  }
  pub fn parsed(&self) -> bool {
    self.flags.get() & Self::PARSED != 0
  }
  pub fn set_parsed(&self) {
    self.flags.set(self.flags.get() | Self::PARSED);
  }
  pub fn shared(&self) -> bool {
    self.flags.get() & Self::SHARED != 0
  }
  /// The name as a string, when it is one (always the case after eval).
  pub fn name_str(&self) -> Option<&'a str> {
    match self.name {
      DeclName::Str(s) => Some(s),
      DeclName::Parts(p) => {
        if p.len() == 1
          && let Node::Keyword(k) = p[0]
        {
          return Some(k.value);
        }
        None
      }
    }
  }
}

/// Mixin parameters / call arguments.
#[derive(Copy, Clone)]
pub struct MixinArg<'a> {
  pub name: Option<&'a str>,
  pub value: Option<Node<'a>>,
  pub expand: bool,
  pub variadic: bool,
}

pub struct MixinCall<'a> {
  pub vis: Vis,
  pub selector: &'a Selector<'a>,
  pub arguments: &'a [MixinArg<'a>],
  pub pos: Pos,
  pub important: bool,
}

/// Extra data for a `MixinDefinition` (which is a `Ruleset` in less.js).
pub struct MixinDef<'a> {
  pub name: &'a str,
  pub params: &'a [MixinArg<'a>],
  pub condition: Option<Node<'a>>,
  pub variadic: bool,
  pub arity: usize,
  pub required: usize,
  pub optional_parameters: &'a [&'a str],
  pub frames: Option<&'a [&'a Ruleset<'a>]>,
}

pub type VarMap<'a> =
  hashbrown::HashMap<&'a str, &'a Declaration<'a>, rustc_hash::FxBuildHasher, &'a Bump>;

/// A scope's `_variables` snapshot: a short list for small scopes (most
/// mixin frames and nested rulesets), a hash map otherwise.
pub enum Vars<'a> {
  Small(&'a [(&'a str, &'a Declaration<'a>)]),
  Map(VarMap<'a>),
}

impl<'a> Vars<'a> {
  pub fn get(&self, name: &str) -> Option<&'a Declaration<'a>> {
    match self {
      // length and last byte first: names share their `@`
      Vars::Small(s) => {
        let (len, last) = (name.len(), name.as_bytes().last());
        s.iter()
          .find(|e| e.0.len() == len && e.0.as_bytes().last() == last && e.0 == name)
          .map(|e| e.1)
      }
      Vars::Map(m) => m.get(name).copied(),
    }
  }
  pub fn names(&self) -> Vec<&'a str> {
    match self {
      Vars::Small(s) => s.iter().map(|e| e.0).collect(),
      Vars::Map(m) => m.keys().copied().collect(),
    }
  }
}
pub type PropMap<'a> =
  hashbrown::HashMap<&'a str, BVec<'a, &'a Declaration<'a>>, rustc_hash::FxBuildHasher, &'a Bump>;

#[derive(Copy, Clone)]
pub struct FoundMixin<'a> {
  pub rule: Node<'a>,
  pub path: &'a [Node<'a>],
}

pub type SelectorPath<'a> = &'a [&'a Selector<'a>];

pub struct RulesetCache<'a> {
  pub variables: RefCell<Option<Vars<'a>>>,
  /// `_properties` (built on a `$prop` lookup; most scopes never need it).
  pub properties: Cell<Option<&'a RefCell<PropMap<'a>>>>,
  /// `_lookups` (allocated on the first mixin lookup through the scope).
  pub lookups: Cell<Option<&'a RefCell<LookupMap<'a>>>>,
  /// Candidate index for mixin lookups in large scopes.
  pub find_index: Cell<Option<&'a FindIndex<'a>>>,
  /// (generation, misses) since the rules last changed shape.
  pub find_misses: Cell<(u32, u32)>,
  /// See [`Ruleset::bump_generation`].
  pub generation: Cell<u32>,
  /// `debugInfo` (only with `dumpLineNumbers`).
  pub debug_info: Cell<Option<&'a DebugInfo<'a>>>,
}

/// Rule indices by the first mixin token of their selectors, for one shape
/// (`generation`) of a rules list; rules replaced in place are added under
/// their new tokens. Candidates are always verified by `selector_match`, so
/// stale entries are harmless.
pub struct FindIndex<'a> {
  pub generation: Cell<u32>,
  pub by_first:
    RefCell<hashbrown::HashMap<&'a str, BVec<'a, u32>, rustc_hash::FxBuildHasher, &'a Bump>>,
  /// Rules whose selectors cannot be indexed statically.
  pub always: RefCell<BVec<'a, u32>>,
}

pub type LookupMap<'a> =
  hashbrown::HashMap<&'a str, &'a [FoundMixin<'a>], rustc_hash::FxBuildHasher, &'a Bump>;

pub struct Ruleset<'a> {
  pub vis: Vis,
  pub selectors: Cell<Option<&'a [&'a Selector<'a>]>>,
  pub rules: RefCell<Option<BVec<'a, Node<'a>>>>,
  pub strict_imports: bool,
  pub root: Cell<bool>,
  pub first_root: Cell<bool>,
  pub allow_imports: Cell<bool>,
  pub multi_media: Cell<bool>,
  pub extend_on_every_path: Cell<bool>,
  /// The variables cache was built and is empty (saves allocating a
  /// `RulesetCache` for the many scopes without variables).
  pub no_variables: Cell<bool>,
  /// Memo for `EvalCtx::is_fixed_point`: 0 unknown, 1 yes, 2 no.
  pub fixed: Cell<u8>,
  /// `originalRuleset`: a ruleset or mixin definition (see [`Ruleset::node`]).
  pub original_ruleset: Cell<Option<&'a Ruleset<'a>>>,
  pub paths: Cell<Option<&'a [SelectorPath<'a>]>>,
  /// Scope caches (`_variables`, `_properties`, `_lookups`), allocated on
  /// first use.
  pub cache: Cell<Option<&'a RulesetCache<'a>>>,
  pub mixin: Option<&'a MixinDef<'a>>,
  pub pos: Pos,
}

impl<'a> Ruleset<'a> {
  /// Bumped whenever `rules` changes shape (see [`FindIndex`]); kept in the
  /// cache, which holds the index.
  pub fn bump_generation(&self) {
    if let Some(c) = self.cache.get() {
      c.generation.set(c.generation.get().wrapping_add(1));
    }
  }
  pub fn generation(&self) -> u32 {
    self.cache.get().map_or(0, |c| c.generation.get())
  }
  pub fn debug_info(&self) -> Option<&'a DebugInfo<'a>> {
    self.cache.get().and_then(|c| c.debug_info.get())
  }
  pub fn set_debug_info(&self, a: Alloc<'a>, d: Option<&'a DebugInfo<'a>>) {
    if d.is_some() || self.cache.get().is_some() {
      self.cache(a).debug_info.set(d);
    }
  }
  pub fn rules_len(&self) -> usize {
    self.rules.borrow().as_ref().map(|r| r.len()).unwrap_or(0)
  }
  pub fn rule_at(&self, i: usize) -> Option<Node<'a>> {
    self.rules.borrow().as_ref().and_then(|r| r.get(i).copied())
  }
  pub fn set_rule(&self, i: usize, n: Node<'a>) {
    if let Some(r) = self.rules.borrow_mut().as_mut() {
      r[i] = n;
    }
  }
  pub fn has_rules(&self) -> bool {
    self.rules.borrow().is_some()
  }
  /// Copy of the current rules (for iteration while the ruleset may change).
  pub fn rules_vec(&self) -> Vec<Node<'a>> {
    self
      .rules
      .borrow()
      .as_ref()
      .map(|r| r.to_vec())
      .unwrap_or_default()
  }
  /// Appends to `paths` (when set).
  pub fn push_paths(&self, a: Alloc<'a>, more: &[SelectorPath<'a>]) {
    if let Some(p) = self.paths.get() {
      let mut v = BVec::with_capacity_in(p.len() + more.len(), a.bump);
      v.extend_from_slice(p);
      v.extend_from_slice(more);
      self.paths.set(Some(v.into_bump_slice()));
    }
  }
  /// The node this ruleset is: a mixin definition when it has mixin data.
  pub fn node(&'a self) -> Node<'a> {
    if self.mixin.is_some() {
      Node::MixinDefinition(self)
    } else {
      Node::Ruleset(self)
    }
  }
  pub fn original(&self) -> Option<Node<'a>> {
    self.original_ruleset.get().map(|r| r.node())
  }
  pub fn reset_cache(&self) {
    self.no_variables.set(false);
    if let Some(c) = self.cache.get() {
      *c.variables.borrow_mut() = None;
      c.properties.set(None);
      if let Some(l) = c.lookups.get() {
        l.borrow_mut().clear();
      }
    }
  }
  #[track_caller]
  pub fn cache(&self, a: Alloc<'a>) -> &'a RulesetCache<'a> {
    match self.cache.get() {
      Some(c) => c,
      None => {
        let c = a.alloc(RulesetCache {
          variables: RefCell::new(None),
          properties: Cell::new(None),
          lookups: Cell::new(None),
          find_index: Cell::new(None),
          find_misses: Cell::new((u32::MAX, 0)),
          generation: Cell::new(0),
          debug_info: Cell::new(None),
        });
        self.cache.set(Some(c));
        c
      }
    }
  }
  pub fn is_mixin(&self) -> bool {
    self.mixin.is_some()
  }
  pub fn selectors(&self) -> &'a [&'a Selector<'a>] {
    self.selectors.get().unwrap_or(&[])
  }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum AtRuleKind {
  AtRule,
  Media,
  Container,
}

pub struct AtRule<'a> {
  pub kind: AtRuleKind,
  pub vis: Vis,
  pub name: &'a str,
  pub value: Option<Node<'a>>,
  pub features: Cell<Option<Node<'a>>>,
  pub rules: Cell<Option<&'a [Node<'a>]>>,
  pub declarations: Cell<Option<&'a [Node<'a>]>>,
  pub simple_block: bool,
  pub is_rooted: bool,
  pub debug_info: Option<DebugInfo<'a>>,
  pub pos: Pos,
  pub evaluated: bool,
}

impl<'a> AtRule<'a> {
  pub fn is_charset(&self) -> bool {
    self.name == "@charset"
  }
  pub fn node(&'a self) -> Node<'a> {
    match self.kind {
      AtRuleKind::AtRule => Node::AtRule(self),
      AtRuleKind::Media => Node::Media(self),
      AtRuleKind::Container => Node::Container(self),
    }
  }
  /// The body ruleset (`rules[0]`) when it is a ruleset.
  pub fn body(&self) -> Option<&'a Ruleset<'a>> {
    self
      .rules
      .get()
      .and_then(|r| r.first())
      .and_then(|n| n.as_ruleset())
  }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct ImportOptions {
  pub less: Option<bool>,
  pub inline: bool,
  pub multiple: Option<bool>,
  pub reference: bool,
  pub optional: bool,
  pub is_plugin: bool,
}

#[derive(Copy, Clone)]
pub enum ImportRoot<'a> {
  Tree(&'a Ruleset<'a>),
  Text(&'a str),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Skip {
  No,
  Yes,
  /// The "once" check that runs on first evaluation.
  Once(u32),
}

pub struct Import<'a> {
  pub vis: Vis,
  pub path: Node<'a>,
  pub features: Option<Node<'a>>,
  pub options: ImportOptions,
  pub pos: Pos,
  pub css: Cell<Option<bool>>,
  pub layer_css: Cell<bool>,
  pub root: Cell<Option<ImportRoot<'a>>>,
  pub imported_filename: Cell<Option<&'a str>>,
  pub imported_file: Cell<Option<u32>>,
  pub skip: Cell<Skip>,
  /// Index into the compilation's deferred import errors.
  pub error: Cell<Option<u32>>,
  /// The evaluated replacement created by the import visitor.
  pub replacement: Cell<Option<&'a Import<'a>>>,
  /// `features` rewritten by the layer() handling in `doEval`.
  pub features_override: Cell<Option<Node<'a>>>,
  /// `path._fileInfo.reference !== undefined`
  pub path_reference: Cell<bool>,
}

impl<'a> Import<'a> {
  pub fn get_path(&self) -> Option<&'a str> {
    match self.path {
      Node::Url(u) => u.value.str_value(),
      n => n.str_value(),
    }
  }
  pub fn features(&self) -> Option<Node<'a>> {
    self.features_override.get().or(self.features)
  }
}

// ---------------------------------------------------------------------------
// Allocation helpers
// ---------------------------------------------------------------------------

/// Arena allocation helpers shared by the parser, evaluator and visitors.
#[derive(Copy, Clone)]
pub struct Alloc<'a> {
  pub bump: &'a Bump,
}

impl<'a> Alloc<'a> {
  #[inline]
  #[track_caller]
  pub fn alloc<T>(&self, v: T) -> &'a T {
    #[cfg(feature = "arena-stats")]
    stats::count::<T>(std::mem::size_of::<T>(), "");
    self.bump.alloc(v)
  }
  #[inline]
  #[track_caller]
  pub fn str(&self, s: &str) -> &'a str {
    #[cfg(feature = "arena-stats")]
    stats::count::<str>(s.len(), "");
    self.bump.alloc_str(s)
  }
  #[inline]
  #[track_caller]
  pub fn slice<T: Copy>(&self, s: &[T]) -> &'a [T] {
    if s.is_empty() {
      return &[];
    }
    #[cfg(feature = "arena-stats")]
    stats::count::<T>(std::mem::size_of_val(s), "[]");
    self.bump.alloc_slice_copy(s)
  }
  #[track_caller]
  pub fn rules(&self, items: &[Node<'a>]) -> &'a RuleList<'a> {
    self.alloc(RuleList {
      items: self.slice(items),
    })
  }
  /// A thin (8-byte) reference to a string; common values are static.
  pub fn vec<T>(&self) -> BVec<'a, T> {
    BVec::new_in(self.bump)
  }
  #[track_caller]
  pub fn vec_from<T: Copy>(&self, s: &[T]) -> BVec<'a, T> {
    #[cfg(feature = "arena-stats")]
    stats::count::<T>(std::mem::size_of_val(s), "vec");
    let mut v = BVec::with_capacity_in(s.len(), self.bump);
    v.extend_from_slice(s);
    v
  }

  #[track_caller]
  pub fn keyword(&self, value: &'a str) -> Node<'a> {
    Node::Keyword(self.alloc(Keyword {
      vis: Vis::default(),
      value,
      no_spacing: Cell::new(None),
      pos: Cell::new(Pos::NONE),
    }))
  }
  #[track_caller]
  pub fn anonymous(&self, value: &'a str, pos: Pos) -> Node<'a> {
    Node::Anonymous(self.anonymous_full(value, pos, false, false, VisInfo::default()))
  }
  #[track_caller]
  pub fn anonymous_full(
    &self,
    value: &'a str,
    pos: Pos,
    map_lines: bool,
    ruleset_like: bool,
    vis: VisInfo,
  ) -> &'a Anonymous<'a> {
    self.alloc(Anonymous {
      vis: Vis::new(vis),
      value,
      pos,
      map_lines,
      ruleset_like,
      prevent_reparse: false,
      no_spacing: Cell::new(None),
      is_null: false,
      has_file: false,
    })
  }
  pub fn null_anonymous(&self) -> Node<'a> {
    Node::Anonymous(self.alloc(Anonymous {
      vis: Vis::default(),
      value: "",
      pos: Pos::NONE,
      map_lines: false,
      ruleset_like: false,
      prevent_reparse: false,
      no_spacing: Cell::new(None),
      is_null: true,
      has_file: false,
    }))
  }
  #[track_caller]
  pub fn expression(&self, value: &[Node<'a>], no_spacing: Option<bool>) -> &'a Expression<'a> {
    self.alloc(Expression {
      vis: Vis::default(),
      value: self.slice(value),
      no_spacing: Cell::new(no_spacing),
      parens: false,
      parens_in_op: Cell::new(false),
      pos: Pos::NONE,
    })
  }
  pub fn value(&self, value: &[Node<'a>]) -> Node<'a> {
    Node::Value(self.alloc(Value {
      vis: Vis::default(),
      value: self.slice(value),
      pos: Pos::NONE,
    }))
  }
  pub fn paren(&self, value: Node<'a>) -> Node<'a> {
    Node::Paren(self.alloc(Paren {
      value,
      no_spacing: Cell::new(None),
    }))
  }
  pub fn quoted(&self, quote: &'a str, value: &'a str, escaped: bool, pos: Pos) -> &'a Quoted<'a> {
    self.alloc(Quoted {
      vis: Vis::default(),
      quote,
      value,
      escaped,
      pos,
      permissive: false,
    })
  }
  pub fn unit(
    &self,
    numerator: &[&'a str],
    denominator: &[&'a str],
    backup: Option<&'a str>,
  ) -> &'a Unit<'a> {
    if numerator.is_empty() && denominator.is_empty() && backup.is_none() {
      // SAFETY of lifetimes: the static empty unit outlives 'a.
      return &EMPTY_UNIT;
    }
    let backup = backup.or_else(|| numerator.first().copied());
    if let Some(u) = common_unit(numerator, denominator, backup) {
      return u;
    }
    let mut n: smallvec::SmallVec<[&'a str; 2]> = numerator.iter().copied().collect();
    n.sort_unstable();
    let mut d: smallvec::SmallVec<[&'a str; 2]> = denominator.iter().copied().collect();
    d.sort_unstable();
    self.alloc(Unit {
      numerator: self.slice(&n),
      denominator: self.slice(&d),
      backup_unit: backup,
    })
  }
  pub fn unit_raw(
    &self,
    numerator: &[&'a str],
    denominator: &[&'a str],
    backup: Option<&'a str>,
  ) -> &'a Unit<'a> {
    if numerator.is_empty() && denominator.is_empty() && backup.is_none() {
      return &EMPTY_UNIT;
    }
    if let Some(u) = common_unit(numerator, denominator, backup) {
      return u;
    }
    self.alloc(Unit {
      numerator: self.slice(numerator),
      denominator: self.slice(denominator),
      backup_unit: backup,
    })
  }
  pub fn dimension(&self, value: f64, unit: &'a Unit<'a>) -> Node<'a> {
    Node::Dimension(self.alloc(Dimension {
      value,
      unit,
      pos: Cell::new(Pos::NONE),
    }))
  }
  pub fn dimension_str(&self, value: f64, unit: &'a str) -> Node<'a> {
    let u = if unit.is_empty() {
      &EMPTY_UNIT
    } else {
      self.unit(&[unit], &[], None)
    };
    self.dimension(value, u)
  }
  pub fn color(&self, rgb: [f64; 3], alpha: f64, value: Option<&'a str>) -> &'a Color<'a> {
    self.alloc(Color {
      rgb,
      alpha,
      value: Cell::new(value),
      pos: Cell::new(Pos::NONE),
    })
  }
  #[track_caller]
  pub fn element(
    &self,
    combinator: Combinator<'a>,
    value: ElValue<'a>,
    is_variable: bool,
    pos: Pos,
    vis: VisInfo,
  ) -> &'a Element<'a> {
    let value = match value {
      ElValue::Str(s) => {
        let b = s.as_bytes();
        // fast path: nothing to trim
        match (b.first(), b.last()) {
          (Some(&f), Some(&l))
            if f.is_ascii()
              && l.is_ascii()
              && !f.is_ascii_whitespace()
              && !l.is_ascii_whitespace() =>
          {
            ElValue::Str(s)
          }
          _ => ElValue::Str(s.trim()),
        }
      }
      v => v,
    };
    self.alloc(Element {
      vis: Vis::new(vis),
      combinator: Cell::new(combinator),
      value,
      is_variable,
      pos,
    })
  }
  /// `new Selector(...)` with an already arena-allocated element list.
  #[track_caller]
  pub fn selector_owned(
    &self,
    elements: &'a [&'a Element<'a>],
    extend_list: Option<&'a [&'a Extend<'a>]>,
    condition: Option<Node<'a>>,
    pos: Pos,
    vis: VisInfo,
  ) -> &'a Selector<'a> {
    self.alloc(Selector::new(
      *self,
      elements,
      condition.map(SelAux::Condition),
      extend_list,
      pos,
      vis,
    ))
  }
  pub fn expression_owned(
    &self,
    value: &'a [Node<'a>],
    no_spacing: Option<bool>,
  ) -> &'a Expression<'a> {
    self.alloc(Expression {
      vis: Vis::default(),
      value,
      no_spacing: Cell::new(no_spacing),
      parens: false,
      parens_in_op: Cell::new(false),
      pos: Pos::NONE,
    })
  }
  pub fn value_owned(&self, value: &'a [Node<'a>]) -> Node<'a> {
    Node::Value(self.alloc(Value {
      vis: Vis::default(),
      value,
      pos: Pos::NONE,
    }))
  }
  #[track_caller]
  pub fn selector(
    &self,
    elements: &[&'a Element<'a>],
    extend_list: Option<&'a [&'a Extend<'a>]>,
    condition: Option<Node<'a>>,
    pos: Pos,
    vis: VisInfo,
  ) -> &'a Selector<'a> {
    let elements: &'a [&'a Element<'a>] = self.slice(elements);
    self.alloc(Selector::new(
      *self,
      elements,
      condition.map(SelAux::Condition),
      extend_list,
      pos,
      vis,
    ))
  }
  /// `new Selector(null)` → a single `&` element.
  pub fn selector_amp(&self, pos: Pos) -> &'a Selector<'a> {
    let el = self.element(
      Combinator::EMPTY,
      ElValue::Str("&"),
      false,
      pos,
      VisInfo::default(),
    );
    self.selector(&[el], None, None, pos, VisInfo::default())
  }
  /// `createEmptySelectors()`.
  pub fn empty_selectors(&self, pos: Pos) -> &'a [&'a Selector<'a>] {
    let s = self.selector_amp(pos);
    s.media_empty.set(true);
    self.slice(&[s])
  }
  #[track_caller]
  pub fn ruleset(
    &self,
    selectors: Option<&'a [&'a Selector<'a>]>,
    rules: Option<BVec<'a, Node<'a>>>,
  ) -> &'a Ruleset<'a> {
    self.ruleset_full(selectors, rules, false, VisInfo::default(), None, Pos::NONE)
  }
  #[track_caller]
  pub fn ruleset_full(
    &self,
    selectors: Option<&'a [&'a Selector<'a>]>,
    rules: Option<BVec<'a, Node<'a>>>,
    strict_imports: bool,
    vis: VisInfo,
    mixin: Option<&'a MixinDef<'a>>,
    pos: Pos,
  ) -> &'a Ruleset<'a> {
    self.alloc(Ruleset {
      vis: Vis::new(vis),
      selectors: Cell::new(selectors),
      rules: RefCell::new(rules),
      strict_imports,
      root: Cell::new(false),
      first_root: Cell::new(false),
      allow_imports: Cell::new(false),
      multi_media: Cell::new(false),
      extend_on_every_path: Cell::new(false),
      no_variables: Cell::new(false),
      fixed: Cell::new(0),
      original_ruleset: Cell::new(None),
      paths: Cell::new(None),
      cache: Cell::new(None),
      mixin,
      pos,
    })
  }
  #[track_caller]
  pub fn declaration(
    &self,
    name: DeclName<'a>,
    value: Node<'a>,
    important: &'a str,
    merge: Merge,
    pos: Pos,
    inline: bool,
    variable: Option<bool>,
  ) -> &'a Declaration<'a> {
    let name: &'a DeclName<'a> = match name {
      DeclName::Str("@arguments") => &DeclName::Str("@arguments"),
      n => self.alloc(n),
    };
    self.declaration_ref(name, value, important, merge, pos, inline, variable)
  }
  #[allow(clippy::too_many_arguments)]
  #[track_caller]
  pub fn declaration_ref(
    &self,
    name: &'a DeclName<'a>,
    value: Node<'a>,
    important: &'a str,
    merge: Merge,
    pos: Pos,
    inline: bool,
    variable: Option<bool>,
  ) -> &'a Declaration<'a> {
    let variable = variable.unwrap_or(matches!(*name, DeclName::Str(s) if s.starts_with('@')));
    self.alloc(Declaration {
      vis: Vis::default(),
      name,
      value: Cell::new(value),
      important: Cell::new(important_code(important)),
      merge: Cell::new(merge),
      pos,
      inline,
      variable,
      flags: Cell::new(0),
    })
  }
  /// A separate copy (not shared) of a declaration.
  pub fn declaration_copy(&self, d: &Declaration<'a>) -> &'a Declaration<'a> {
    self.alloc(Declaration {
      vis: Vis::new(d.vis.info()),
      name: d.name,
      value: Cell::new(d.value.get()),
      important: Cell::new(d.important.get()),
      merge: Cell::new(d.merge.get()),
      pos: d.pos,
      inline: d.inline,
      variable: d.variable,
      flags: Cell::new(d.flags.get() & !Declaration::SHARED),
    })
  }
  pub fn detached_ruleset(
    &self,
    ruleset: Node<'a>,
    frames: Option<&'a [&'a Ruleset<'a>]>,
  ) -> Node<'a> {
    Node::DetachedRuleset(self.alloc(DetachedRuleset {
      vis: Vis::default(),
      ruleset,
      frames,
    }))
  }
  pub fn variable(&self, name: &'a str, pos: Pos) -> Node<'a> {
    Node::Variable(self.alloc(Variable {
      name,
      pos,
      evaluating: Cell::new(false),
      no_spacing: Cell::new(None),
    }))
  }
  pub fn call(&self, name: &'a str, args: &[Node<'a>], pos: Pos) -> Node<'a> {
    Node::Call(self.alloc(Call {
      vis: Vis::default(),
      name,
      args: self.slice(args),
      pos,
      calc: name == "calc",
    }))
  }
}

/// Arena accounting by type and call site (feature `arena-stats`).
#[cfg(feature = "arena-stats")]
pub mod stats {
  use std::{cell::RefCell, collections::HashMap};

  use cow_utils::CowUtils;
  thread_local! {
      static COUNTS: RefCell<HashMap<(&'static str, &'static str, u32, &'static str), (usize, usize)>> = RefCell::new(HashMap::new());
  }
  #[track_caller]
  pub fn count<T: ?Sized>(bytes: usize, kind: &'static str) {
    let l = std::panic::Location::caller();
    COUNTS.with(|c| {
      let mut c = c.borrow_mut();
      let e = c
        .entry((std::any::type_name::<T>(), l.file(), l.line(), kind))
        .or_insert((0, 0));
      e.0 += 1;
      e.1 += bytes;
    });
  }
  /// Prints and resets the counts.
  pub fn dump(label: &str) {
    COUNTS.with(|c| {
      let mut v: Vec<_> = c.borrow_mut().drain().collect();
      v.sort_by_key(|x| std::cmp::Reverse(x.1.1));
      let total: usize = v.iter().map(|x| x.1.1).sum();
      eprintln!("== {} tracked {} KB", label, total / 1024);
      for ((t, f, line, kind), (n, b)) in v.iter().take(30) {
        let t = t
          .cow_replace("less_rs::tree::", "")
          .cow_replace("'_", "")
          .into_owned();
        let f = f.rsplit("src/").next().unwrap_or(f);
        eprintln!("{:>8} {:>7}KB {}{} @{}:{}", n, b / 1024, kind, t, f, line);
      }
    });
  }
}

/// Normalises a `!important` spelling to less.js's `" " + important.trim()`.
pub fn important_str<'a>(a: Alloc<'a>, important: &str) -> &'a str {
  let t = important.trim();
  match t {
    "" if important.is_empty() => "",
    "!important" => " !important",
    _ => {
      let mut s = bumpalo::collections::String::with_capacity_in(t.len() + 1, a.bump);
      s.push(' ');
      s.push_str(t);
      s.into_bump_str()
    }
  }
}

static NEXT_EXTEND_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

pub fn next_extend_id() -> u32 {
  NEXT_EXTEND_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

impl<'a> Alloc<'a> {
  pub fn extend(
    &self,
    selector: &'a Selector<'a>,
    option: Option<&'a str>,
    pos: Pos,
    vis: VisInfo,
  ) -> &'a Extend<'a> {
    let id = next_extend_id();
    let all = matches!(option, Some("all") | Some("!all"));
    self.alloc(Extend {
      vis: Vis::new(vis),
      selector,
      option,
      pos,
      object_id: id,
      parent_ids: RefCell::new({
        let mut v = BVec::new_in(self.bump);
        v.push(id);
        v
      }),
      allow_before: all,
      allow_after: all,
      self_selectors: Cell::new(&[]),
      ruleset: Cell::new(None),
      first_extend_on_this_selector_path: Cell::new(false),
      has_found_matches: Cell::new(false),
    })
  }
}
