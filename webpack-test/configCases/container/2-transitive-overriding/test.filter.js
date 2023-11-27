const { FilteredStatus } = require("../../../lib/util/filterUtil")

module.exports = () => {return [FilteredStatus.PARTIAL_PASS, "Rspack unshift entry to add MF related runtime, so the snapshot of modules is different, need to measure wether to align the default runtime with webpack"]}