use bitflags::bitflags;
use swc_atoms::JsWord;
use swc_common::SyntaxContext;
use swc_ecma_ast::Id;
use ustr::Ustr;

bitflags! {
    pub struct SymbolFlag: u32 {
        const DEFAULT =  1 << 0;
        const EXPORT =  1 << 1;
        const VAR_DECL = 1 << 2;
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndirectTopLevelSymbol {
  pub uri: Ustr,
  pub id: JsWord,
}

impl IndirectTopLevelSymbol {
  pub fn new(uri: Ustr, id: JsWord) -> Self {
    Self { uri, id }
  }

  pub fn uri(&self) -> Ustr {
    self.uri
  }

  pub fn id(&self) -> &str {
    self.id.as_ref()
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
