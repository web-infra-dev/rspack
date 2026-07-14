function assert(condition, message) {
  if (!condition) throw new Error(`Assertion failed: ${message}`);
}

module.exports = {
  checkStats(stepName, _, stats) {
    if (stepName === "0") {
      assert(
        stats.includes("<t> rebuild chunk graph"),
        "cold build must build the chunk graph",
      );
    } else if (stepName === "1") {
      assert(
        !stats.includes("<t> rebuild chunk graph"),
        "editing an existing leaf must reuse the chunk graph",
      );
      assert(
        !stats.includes("new module detected"),
        "an existing leaf must not be treated as a new module",
      );
    } else {
      throw new Error(`Unexpected watch step: ${stepName}`);
    }
    return true;
  },
};
