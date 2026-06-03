module.exports = {
  noTests: true,
  findBundle() {
    return ["main.mjs", "other.mjs"];
  },
  snapshotContent(content) {
    return content
      .replaceAll(
        "/*! <TEST_ROOT>/esmOutputCases/dynamic-import/import-context-multi-entry/modules-a/ */",
        "/*! ./modules-a/ */",
      )
      .replaceAll(
        "/*! <TEST_ROOT>/esmOutputCases/dynamic-import/import-context-multi-entry/modules-b/ */",
        "/*! ./modules-b/ */",
      )
      .replace(/[ \t]+$/gm, "");
  },
};
