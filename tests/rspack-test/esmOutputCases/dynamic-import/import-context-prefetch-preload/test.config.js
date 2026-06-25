module.exports = {
  snapshotContent(content) {
    return content
      .replaceAll(
        "/*! <TEST_ROOT>/esmOutputCases/dynamic-import/import-context-prefetch-preload/modules/ */",
        "/*! ./modules/ */",
      )
      .replace(/[ \t]+$/gm, "");
  },
};
