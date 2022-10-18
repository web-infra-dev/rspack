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
#[derive(Debug, Hash)]
pub(crate) struct Symbol {
  pub(crate) uri: Ustr,
  ctxt: SyntaxContext,
  atom: JsWord,
}

impl Symbol {
  pub(crate) fn new(uri: ustr::Ustr, ctxt: SyntaxContext, atom: JsWord) -> Self {
    Self { uri, ctxt, atom }
  }

  pub(crate) fn from_id_and_uri(id: Id, uri: Ustr) -> Self {
    let (atom, ctxt) = id;
    Self { atom, ctxt, uri }
  }
}

#[derive(Debug)]
pub(crate) struct IndirectTopLevelSymbol {
  uri: Ustr,
  id: JsWord,
}

impl IndirectTopLevelSymbol {
  pub(crate) fn new(uri: Ustr, id: JsWord) -> Self {
    Self { uri, id }
  }
}
