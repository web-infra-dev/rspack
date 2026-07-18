function assert(condition, message) {
  if (!condition) throw new Error(`Assertion failed: ${message}`);
}

module.exports = {
  checkStats(stepName, _, stats) {
    const rebuild = stats.includes("<t> rebuild chunk graph");
    if (stepName === "0") {
      assert(rebuild, "cold build must build the chunk graph");
    } else if (stepName === "1") {
      assert(
        !rebuild,
        "editing the leaf must reuse stable sync and async topology",
      );
      assert(
        !stats.includes("module outgoings change detected"),
        "root and async-block outgoings must be compared independently",
      );
    } else if (stepName === "2") {
      assert(
        rebuild,
        "changing an async chunk name must rebuild the chunk graph",
      );
    } else if (stepName === "3") {
      assert(rebuild, "retargeting an async edge must rebuild the chunk graph");
    } else if (stepName === "4") {
      assert(!rebuild, "moving stable async edges must reuse the chunk graph");
      assert(
        !stats.includes("module async blocks change detected"),
        "source-location changes must not invalidate stable async blocks",
      );
    } else if (stepName === "5") {
      assert(rebuild, "inserting an async edge must rebuild the chunk graph");
    } else if (stepName === "6") {
      assert(rebuild, "removing an async edge must rebuild the chunk graph");
    } else if (stepName === "7") {
      assert(
        !rebuild,
        "editing a module with no outgoing edges must reuse the chunk graph",
      );
      assert(
        !stats.includes("new module detected"),
        "an existing terminal module must not be treated as a new module",
      );
    } else {
      throw new Error(`Unexpected watch step: ${stepName}`);
    }
    return true;
  },
};
