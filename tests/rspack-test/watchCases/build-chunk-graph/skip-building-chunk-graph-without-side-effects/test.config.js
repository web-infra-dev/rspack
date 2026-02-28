function assert(condition, message) {
	if (!condition) {
		throw new Error(`Assertion failed for ${message}`);
	}
}

function should_rebuild(stats) {
	return stats.includes("<t> rebuild chunk graph");
}

module.exports = {
	checkStats(stepName, _, stats) {
		switch (stepName) {
			case "0":
				assert(should_rebuild(stats), "should rebuild chunk graph");
				break;
			case "1":
				assert(!should_rebuild(stats), "should not rebuild chunk graph");
				break;
			default:
				throw "not have more step";
		}

		return true;
	}
};
