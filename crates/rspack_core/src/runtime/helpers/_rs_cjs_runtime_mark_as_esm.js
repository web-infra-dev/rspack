// treat exports as esm module
function __mark_as_esm__(exports) {
  // ported from webpack, but are these two lines really needed?
  // since I didn't see any library depends on this.
  if (typeof Symbol !== 'undefined' && Symbol.toStringTag) {
    Object.defineProperty(exports, Symbol.toStringTag, { value: 'Module' });
  }
  Object.defineProperty(exports, '__esModule', { value: true });
}

globalThis.rs.mark_as_esm = globalThis.rs.mark_as_esm || __mark_as_esm__;
