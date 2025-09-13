const { FilteredStatus } = require("@rspack/test-tools/helper/util/filterUtil");

module.exports = () => [
  FilteredStatus.PARTIAL_PASS,
  "FIXME: ident name not contain loader path"
];
