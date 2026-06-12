const markers = [
  "PURE_NESTING_FN_A_MARKER",
  "PURE_NESTING_FN_B_MARKER",
  "DEAD_COND_ALT_MARKER",
  "DEAD_COND_CONS_MARKER",
  "DEAD_LOGICAL_AND_MARKER",
  "DEAD_LOGICAL_OR_MARKER",
  "DEAD_NULLISH_MARKER",
];
const retainedMarkers = [
  "UNSAFE_OPTIONAL_MEMBER_MARKER",
  "UNSAFE_SHORTHAND_MARKER",
  "UNSAFE_UNARY_PURE_CALL_MARKER",
  "SHADOWED_PURE_PARAM_MARKER",
  "SHADOWED_PURE_LOCAL_MARKER",
  "SHADOWED_PURE_FN_EXPR_NAME_MARKER",
];

module.exports = {
  snapshotContent(content) {
    for (const marker of markers) {
      if (content.includes(marker)) {
        throw new Error(`Expected pure function marker ${marker} to be removed from ESM output`);
      }
    }
    for (const marker of retainedMarkers) {
      if (!content.includes(marker)) {
        throw new Error(`Expected side-effect marker ${marker} to be preserved in ESM output`);
      }
    }
    if (
      !content.includes(
        'null && (pureWithLocalMarkedDeclaration("LOCAL_MARKED_DECL_MARKER"))'
      )
    ) {
      throw new Error("Expected local marked declaration call to be shaken in ESM output");
    }
    return content;
  },
};
