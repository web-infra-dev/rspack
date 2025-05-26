var supportsWebAssembly = require("../../../helpers/supportsWebAssembly");
var supportsResponse = require("../../../helpers/supportsResponse");
const { FilteredStatus } = require("../../../lib/util/filterUtil");
module.exports = function (config) {
  if (supportsWebAssembly() && supportsResponse()) {
    return [FilteredStatus.PARTIAL_PASS, "TODO: support sync wasm"];
  }
  return false;
};
