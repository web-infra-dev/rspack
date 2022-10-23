use bitflags::bitflags;
use swc_atoms::JsWord;
use swc_common::SyntaxContext;
use swc_ecma_ast::Id;
use ustr::Ustr;

bitflags! {
    pub(crate) struct SymbolFlag: u32 {
        const DEFAULT_EXPORT =  1 << 0;
        const USED = 1 << 1;
    }

}
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct Symbol {
  pub(crate) uri: Ustr,
  pub(crate) id: BetterId,
}

impl Symbol {
  pub(crate) fn new(uri: ustr::Ustr, ctxt: SyntaxContext, atom: JsWord) -> Self {
    Self {
      uri,
      id: (atom, ctxt).into(),
    }
  }

  pub(crate) fn from_id_and_uri(id: BetterId, uri: Ustr) -> Self {
    Self { uri, id }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct IndirectTopLevelSymbol {
  pub(crate) uri: Ustr,
  pub(crate) id: JsWord,
}

impl IndirectTopLevelSymbol {
  pub(crate) fn new(uri: Ustr, id: JsWord) -> Self {
    Self { uri, id }
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
