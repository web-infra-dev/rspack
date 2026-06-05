const markers = ["PURE_NESTING_FN_A_MARKER", "PURE_NESTING_FN_B_MARKER"];

module.exports = {
  snapshotContent(content) {
    for (const marker of markers) {
      if (content.includes(marker)) {
        throw new Error(`Expected pure function marker ${marker} to be removed from ESM output`);
      }
    }
    return content;
  },
};
