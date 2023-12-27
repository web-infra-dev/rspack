const { FilteredStatus } = require("../../../lib/util/filterUtil")
// this test depend on 1-container-full
module.exports = () => {return [FilteredStatus.PARTIAL_PASS, "https://github.com/web-infra-dev/rspack/issues/4784"]}
