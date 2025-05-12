var supportsOptionalChaining = require("../../../helpers/supportsOptionalChaining");
const { FilteredStatus } = require("../../../lib/util/filterUtil");

module.exports = function (config) {
  if (supportsOptionalChaining()) {
    return [
      FilteredStatus.PARTIAL_PASS,
      "should not evaluate new RegExp for redefined RegExp"
    ]
  } else {
    return false;
  }
};

