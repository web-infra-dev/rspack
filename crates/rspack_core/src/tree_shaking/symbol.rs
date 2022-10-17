use bitflags::bitflags;
use swc_atoms::JsWord;
use swc_common::SyntaxContext;

bitflags! {
    pub(crate) struct SymbolFlag: u32 {
        const DEFAULT_EXPORT =  1 << 0;
        const USED = 1 << 1;
    }

}
#[derive(Debug, Hash)]
pub(crate) struct Symbol {
  pub(crate) uri: ustr::Ustr,
  ctxt: SyntaxContext,
  atom: JsWord,
}
