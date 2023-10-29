const { FilteredStatus } = require("../../../lib/util/filterUtil")

module.exports = () => [FilteredStatus.PARTIAL_PASS, 'not have the same name for splitted chunk with webpack https://github.com/web-infra-dev/rspack/issues/4334']
