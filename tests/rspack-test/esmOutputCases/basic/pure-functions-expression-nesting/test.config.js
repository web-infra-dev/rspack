const removedMarkers = [
  "PURE_NESTING_ARRAY_A_MARKER",
  "PURE_NESTING_ARRAY_B_MARKER",
  "PURE_NESTING_LOCAL_A_MARKER",
  "PURE_NESTING_LOCAL_B_MARKER",
];

module.exports = {
  snapshotContent(content) {
    for (const marker of removedMarkers) {
      if (content.includes(marker)) {
        throw new Error(`Expected pure function marker ${marker} to be removed from ESM output`);
      }
    }
    return content;
  },
};
