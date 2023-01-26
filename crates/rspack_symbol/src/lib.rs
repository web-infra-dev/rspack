use bitflags::bitflags;
use swc_core::ecma::ast::Id;
use swc_core::ecma::atoms::JsWord;
use swc_core::{common::SyntaxContext, ecma::atoms::js_word};
use ustr::Ustr;

bitflags! {
    pub struct SymbolFlag: u8 {
        const DEFAULT =  1 << 0;
        const EXPORT =  1 << 1;
        const VAR_DECL = 1 << 2;
        const ARROW_EXPR = 1 << 3;
        const FUNCTION_EXPR = 1 << 4;
        const CLASS_EXPR = 1 << 5;
        const ALIAS = 1 << 6;
        const EXPORT_DEFAULT = Self::DEFAULT.bits | Self::EXPORT.bits;
    }
}
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct Symbol {
  pub(crate) uri: Ustr,
  pub(crate) id: BetterId,
  pub(crate) ty: SymbolType,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq, Copy)]
pub enum SymbolType {
  Define,
  Temp,
}

impl Symbol {
  pub fn new(uri: Ustr, id: BetterId, ty: SymbolType) -> Self {
    Self { uri, id, ty }
  }

  pub fn uri(&self) -> Ustr {
    self.uri
  }

  pub fn id(&self) -> &BetterId {
    &self.id
  }

  pub fn ty(&self) -> &SymbolType {
    &self.ty
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum IndirectType {
  Temp(JsWord),
  /// first argument is original, second argument is exported
  ReExport(JsWord, Option<JsWord>),
  /// first argument is local, second argument is imported
  Import(JsWord, Option<JsWord>),
  ///
  ImportDefault(JsWord),
}
pub static default_js_word: JsWord = js_word!("default");
/// We have three kind of star symbol
/// ## import with namespace
/// ```js
/// // a.js
/// import * as xx './b.js'
/// // this generate a a `StarSymbol` like
/// ```
/// ```rs,no_run
/// StarSymbol {
///   src: "./b.js",
///   binding: "xx",
///   reexporter: ""
/// }
/// ```
/// ##  reexport all
/// ```js
/// // a.js
/// export * from './b.js'
/// // this generate a a `StarSymbol` like
/// ```
/// ```rs,no_run
/// StarSymbol {
///   src: "./b.js",
///   binding: "",
///   reexporter: "a.js"
/// }
/// ```
/// ##  reexport * with a binding
/// ```js
/// // a.js
/// export * as something from './b.js'
/// // this generate a a `StarSymbol` like
/// ```
/// ```rs,no_run
/// StarSymbol {
///   src: "./b.js",
///   binding: "something",
///   reexporter: "a.js"
/// }
/// ```
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct StarSymbol {
  pub src: Ustr,
  pub binding: JsWord,
  pub module_ident: Ustr,
  pub ty: StarSymbolKind,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum StarSymbolKind {
  ReExportAllAs,
  ImportAllAs,
  ReExportAll,
}

impl StarSymbol {
  pub fn star_kind(&self) {}
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct IndirectTopLevelSymbol {
  pub src: Ustr,
  pub ty: IndirectType,
  // module identifier of module that import me, only used for debugging
  pub importer: Ustr,
}

// impl std::cmp::PartialEq for IndirectTopLevelSymbol {
//   fn eq(&self, other: &Self) -> bool {
//     self.uri == other.uri
//       && self.id == other.id
//       && (self.importer == other.importer || self.importer.is_empty())
//   }
// }

impl IndirectTopLevelSymbol {
  pub fn new(src: Ustr, importer: Ustr, ty: IndirectType) -> Self {
    Self { src, importer, ty }
  }

  /// if `self.ty == IndirectType::Rexport`, it return `exported` if it is [Some] else it return `original`.
  /// else it return binding
  pub fn indirect_id(&self) -> &JsWord {
    match self.ty {
      IndirectType::Temp(ref ident) => ident,
      IndirectType::ReExport(ref original, ref exported) => match exported {
        Some(exported) => exported,
        None => original,
      },
      IndirectType::Import(ref local, ref imported) => match imported {
        Some(imported) => imported,
        None => local,
      },
      // we store the binding just used for create [ModuleDecl], but it always to DefaultExport of some module
      IndirectType::ImportDefault(_) => &default_js_word,
    }
  }

  pub fn id(&self) -> &JsWord {
    match self.ty {
      IndirectType::Temp(ref ident) => ident,
      IndirectType::ReExport(ref original, ref exported) => match exported {
        Some(exported) => exported,
        None => original,
      },
      IndirectType::Import(ref local, ref _imported) => local,
      IndirectType::ImportDefault(_) => &default_js_word,
    }
  }

  // pub fn fast_create(uri: Ustr, id: JsWord) -> IndirectTopLevelSymbol {
  //   IndirectTopLevelSymbol {
  //     uri,
  //     id,
  //     importer: ustr(""),
  //     ty: Default::default(),
  //   }
  // }

  pub fn src(&self) -> Ustr {
    self.src
  }

  pub fn importer(&self) -> Ustr {
    self.importer
  }

  pub fn is_reexport(&self) -> bool {
    matches!(&self.ty, IndirectType::ReExport(_, _))
  }

  pub fn is_temp(&self) -> bool {
    matches!(&self.ty, IndirectType::Temp(_))
  }

  pub fn is_import(&self) -> bool {
    matches!(&self.ty, IndirectType::Import(_, _))
  }
}

/// Just a wrapper type of [swc_ecma_ast::Id],just want a better debug experience e.g.
/// `BetterId.debug()` -> `xxxxxxx|#10`
/// debug of [swc_ecma_ast::Id] -> `(#1, atom: Atom('b' type=static))`
/// We don't care the kind of inter of the [JsWord]
#[derive(Hash, Clone, PartialEq, Eq, Default)]
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
