function assert(condition, message) {
  if (!condition) throw new Error(`Assertion failed: ${message}`);
}

module.exports = {
  findBundle(_, __, stepName) {
    return stepName === "3" ? "renamed.js" : "main.js";
  },
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
        "an existing terminal module must not be treated as a new module",
      );
    } else if (stepName === "2") {
      assert(
        stats.includes("<t> rebuild chunk graph"),
        "changing a global entry root must rebuild the chunk graph",
      );
      assert(
        stats.includes("entry data change detected"),
        "the changed global entry root must invalidate the cached chunk graph",
      );
    } else if (stepName === "3") {
      assert(
        stats.includes("<t> rebuild chunk graph"),
        "changing entry options must rebuild the chunk graph",
      );
      assert(
        stats.includes("entry data change detected"),
        "changed entry options must invalidate the cached chunk graph",
      );
    } else {
      throw new Error(`Unexpected watch step: ${stepName}`);
    }
    return true;
  },
};
