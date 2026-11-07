function assert(condition, message) {
	if (!condition) {
		throw new Error(`Assertion failed: ${message}`);
	}
}

export default {
	checkStats(stepName, _, stats) {
		assert(
			stats.includes("<t> rebuild chunk graph"),
			"a new side effect must rebuild the chunk graph"
		);
		if (stepName === "1") {
			assert(
				stats.includes("module topology change detected"),
				"a new side effect must fail the topology guard"
			);
		}
		return true;
	}
};
