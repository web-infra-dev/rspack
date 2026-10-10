// The webworker harness can't run this two-entry split-chunk setup (fails with or without force-loading)
module.exports = (options) => options.target !== "webworker";
