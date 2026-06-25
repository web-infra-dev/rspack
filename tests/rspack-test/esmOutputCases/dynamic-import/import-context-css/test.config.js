module.exports = {
  noTests: true,
  snapshotFileFilter(file) {
    return file.endsWith(".mjs") || file.endsWith(".css");
  },
  snapshotContent(content) {
    return content
      .replaceAll(
        "/*! <TEST_ROOT>/esmOutputCases/dynamic-import/import-context-css/modules/ */",
        "/*! ./modules/ */",
      )
      .replace(/[ \t]+$/gm, "");
  },
};
