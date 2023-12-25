const { FilteredStatus } = require("../../../lib/util/filterUtil")

module.exports = () => {return [FilteredStatus.PARTIAL_PASS, "Rspack has an extra empty chunk"]}