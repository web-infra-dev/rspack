module.exports = {
  snapshotContent(content) {
    return content
      .replaceAll(
        "/*! <TEST_ROOT>/esmOutputCases/dynamic-import/import-context-multi-chunk/modules/ */",
        "/*! ./modules/ */",
      )
      .replace(/[ \t]+$/gm, "");
  },
};
