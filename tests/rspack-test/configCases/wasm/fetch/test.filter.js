const { FilteredStatus } = require("@rspack/test-tools/helper/util/filterUtil");
module.exports = function (config) {
  return [FilteredStatus.PARTIAL_PASS, "TODO: support sync wasm"];
};
