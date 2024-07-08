const { FilteredStatus } = require("../../../lib/util/filterUtil")

module.exports = () => {
	return [
		FilteredStatus.PARTIAL_PASS,
		"https://github.com/web-infra-dev/rspack/issues/4923"
	]
}
