module.exports = {
  snapshotContent(content) {
    if (!content.includes("OPTIONAL_CALL_WITH_PURE_FUNCTIONS_DISABLED")) {
      throw new Error("Expected optional call to be preserved when pureFunctions is disabled");
    }
    return content;
  },
};
