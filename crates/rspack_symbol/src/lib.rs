use bitflags::bitflags;
use swc_core::common::SyntaxContext;
use swc_core::ecma::ast::Id;
use swc_core::ecma::atoms::JsWord;
use ustr::{ustr, Ustr};

bitflags! {
    pub struct SymbolFlag: u8 {
        const DEFAULT =  1 << 0;
        const EXPORT =  1 << 1;
        const VAR_DECL = 1 << 2;
        const ARROW_EXPR = 1 << 3;
        const FUNCTION_EXPR = 1 << 4;
        const EXPORT_DEFAULT = Self::DEFAULT.bits | Self::EXPORT.bits;
    }
}
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct Symbol {
  pub(crate) uri: Ustr,
  pub(crate) id: BetterId,
}

impl Symbol {
  pub fn from_id_and_uri(id: BetterId, uri: Ustr) -> Self {
    Self { uri, id }
  }

  pub fn uri(&self) -> Ustr {
    self.uri
  }

  pub fn id(&self) -> &BetterId {
    &self.id
  }
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub enum IndirectType {
  #[default]
  Default,
  ReExport,
}

#[derive(Debug, Clone, Eq)]
pub struct IndirectTopLevelSymbol {
  pub uri: Ustr,
  pub id: JsWord,
  pub ty: IndirectType,
  // module identifier of module that import me, only used for debugging
  importer: Ustr,
}

impl std::hash::Hash for IndirectTopLevelSymbol {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.uri.hash(state);
    self.id.hash(state);
  }
}
impl std::cmp::PartialEq for IndirectTopLevelSymbol {
  fn eq(&self, other: &Self) -> bool {
    self.uri == other.uri && self.id == other.id
  }
}

impl IndirectTopLevelSymbol {
  pub fn new(uri: Ustr, id: JsWord, importer: Ustr, ty: IndirectType) -> Self {
    Self {
      uri,
      id,
      importer,
      ty,
    }
  }

  pub fn from_uri_and_id(uri: Ustr, id: JsWord) -> IndirectTopLevelSymbol {
    // Because importer don't affect hash result so empty `Ustr` is alright here.
    IndirectTopLevelSymbol {
      uri,
      id,
      importer: ustr(""),
      ty: Default::default(),
    }
  }

  pub fn uri(&self) -> Ustr {
    self.uri
  }

  pub fn id(&self) -> &str {
    self.id.as_ref()
  }

  pub fn importer(&self) -> Ustr {
    self.importer
  }
}

/// Just a wrapper type of [swc_ecma_ast::Id],just want a better debug experience e.g.
/// `BetterId.debug()` -> `xxxxxxx|#10`
/// debug of [swc_ecma_ast::Id] -> `(#1, atom: Atom('b' type=static))`
/// We don't care the kind of inter of the [JsWord]
#[derive(Hash, Clone, PartialEq, Eq)]
pub struct BetterId {
  pub ctxt: SyntaxContext,
  pub atom: JsWord,
}

impl std::fmt::Debug for BetterId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "'{}'#{}", &self.atom, self.ctxt.as_u32())
  }
}

impl From<Id> for BetterId {
  fn from(id: Id) -> Self {
    let (atom, ctxt) = id;
    Self { ctxt, atom }
  }
}

#[derive(Debug, Eq, Clone)]
pub struct SymbolExt {
  pub id: BetterId,
  pub flag: SymbolFlag,
}

impl SymbolExt {
  pub fn flag(&self) -> SymbolFlag {
    self.flag
  }

  pub fn id(&self) -> &BetterId {
    &self.id
  }

  pub fn new(id: BetterId, flag: SymbolFlag) -> SymbolExt {
    SymbolExt { id, flag }
  }

  pub fn set_id(&mut self, id: BetterId) {
    self.id = id;
  }

  pub fn set_flag(&mut self, flag: SymbolFlag) {
    self.flag = flag;
  }
}

/// We only hash `BetterId`, because the flag is gone after the first time
/// insertion, we can only get the `BetterId` from swc ast. Same for other trait
impl std::hash::Hash for SymbolExt {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.id.hash(state);
  }
}

impl From<BetterId> for SymbolExt {
  fn from(id: BetterId) -> Self {
    Self {
      id,
      flag: SymbolFlag::empty(),
    }
  }
}

impl PartialEq for SymbolExt {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

/// This enum hold a `Id` from `swc` or a simplified member expr with a `Id` and a `JsWord`
/// This is useful when we want to tree-shake the namespace access property e.g.
/// assume we have
///
/// **a.js**
/// ```js
/// export function a() {}
/// export function b() {}
/// ```
/// **b.js**
/// ```js
/// improt * as a from './a.js'
/// a.a()
/// ```
/// In such scenario only `a` from `a.js` is used, `a.b` is unused.
/// We use [BetterIdOrMemExpr::MemberExpr] to represent namespace access
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum IdOrMemExpr {
  Id(BetterId),
  MemberExpr { object: BetterId, property: JsWord },
}

impl IdOrMemExpr {
  /// Returns `true` if the better id or mem expr is [`Id`].
  ///
  /// [`Id`]: BetterIdOrMemExpr::Id
  #[must_use]
  pub fn is_id(&self) -> bool {
    matches!(self, Self::Id(..))
  }

  /// Returns `true` if the better id or mem expr is [`MemberExpr`].
  ///
  /// [`MemberExpr`]: BetterIdOrMemExpr::MemberExpr
  #[must_use]
  pub fn is_member_expr(&self) -> bool {
    matches!(self, Self::MemberExpr { .. })
  }

  pub fn get_id(&self) -> &BetterId {
    match self {
      IdOrMemExpr::Id(id) => id,
      IdOrMemExpr::MemberExpr { object, .. } => object,
    }
  }
}
