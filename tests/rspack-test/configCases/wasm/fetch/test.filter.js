var supportsWebAssembly = require("@rspack/test-tools/helper/legacy/supportsWebAssembly");
var supportsResponse = require("@rspack/test-tools/helper/legacy/supportsResponse");
const { FilteredStatus } = require("@rspack/test-tools/helper/util/filterUtil");
module.exports = function (config) {
  if (supportsWebAssembly() && supportsResponse()) {
    return [FilteredStatus.PARTIAL_PASS, "TODO: support sync wasm"];
  }
  return false;
};
