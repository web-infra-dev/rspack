module.exports = {
  snapshotContent(content) {
    return content
      .replaceAll(
        "/*! <TEST_ROOT>/esmOutputCases/dynamic-import/import-context-lazy/modules/ */",
        "/*! ./modules/ */",
      )
      .replace(/[ \t]+$/gm, "");
  },
};
