var supportsOptionalChaining = require("@rspack/test-tools/helper/legacy/supportsOptionalChaining");
const { FilteredStatus } = require("@rspack/test-tools/helper/util/filterUtil");

module.exports = function (config) {
  // if (supportsOptionalChaining()) {
  //   return [
  //     FilteredStatus.PARTIAL_PASS,
  //     "TODO: not support moduleIds: 'size'"
  //   ]
  // } else {
  return "FIXME: timeout on CI";
  // }
};

