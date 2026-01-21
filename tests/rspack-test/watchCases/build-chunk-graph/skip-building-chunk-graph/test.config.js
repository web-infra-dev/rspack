function assert(condition, message) {
	if (!condition) {
    console.log("failed with message:", message);
		throw new Error("Assertion failed for");
	}
}

function should_rebuild(stats) {
	return stats.includes("<t> rebuild chunk graph");
}

module.exports = {
	checkStats(stepName, _, stats) {
		switch (stepName) {
			case "0":
				assert(should_rebuild(stats), stats);
				break;
			case "1":
				assert(!should_rebuild(stats, stats));
				break;
			case "2":
				assert(should_rebuild(stats), stats);
				break;
			case "3":
				assert(!should_rebuild(stats), stats);
				break;
			case "4":
				assert(should_rebuild(stats), stats);
				break;
			default:
				throw "not have more step";
		}

		return true;
	}
};
