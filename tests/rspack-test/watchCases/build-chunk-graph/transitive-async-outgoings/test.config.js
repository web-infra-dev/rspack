function assert(condition, message) {
	if (!condition) {
		throw new Error(`Assertion failed: ${message}`);
	}
}

module.exports = {
	checkStats(stepName, _, stats) {
		if (stepName === "0") {
			assert(
				stats.includes("<t> rebuild chunk graph"),
				"cold build must build the chunk graph"
			);
		} else if (stepName === "1") {
			assert(
				!stats.includes("<t> rebuild chunk graph"),
				"a stable transitive async topology must reuse the chunk graph"
			);
			assert(
				!stats.includes("module topology change detected"),
				"stable async outgoings must pass the topology guard"
			);
		} else if (stepName === "2") {
			assert(
				stats.includes("<t> rebuild chunk graph"),
				"changing async chunk options must rebuild the chunk graph"
			);
			assert(
				stats.includes("module topology change detected"),
				"the topology guard must reject changed async chunk options"
			);
		} else {
			throw new Error(`Unexpected watch step: ${stepName}`);
		}
		return true;
	}
};
