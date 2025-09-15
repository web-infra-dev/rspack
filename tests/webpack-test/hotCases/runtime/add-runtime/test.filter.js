var supportsWorker = require("@rspack/test-tools/helper/legacy/supportsWorker");

module.exports = function (config) {
  if (config.target !== "web") {
    return false;
  }
  return supportsWorker();
};
