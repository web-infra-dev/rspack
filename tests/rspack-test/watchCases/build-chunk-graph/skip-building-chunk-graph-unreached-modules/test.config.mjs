function assert(condition, message) {
	if (!condition) {
		throw new Error(`Assertion failed: ${message}`);
	}
}

export default {
	checkStats(stepName, _, stats) {
		if (stepName === "0") {
			assert(
				stats.includes("<t> rebuild chunk graph"),
				"cold build must build the chunk graph"
			);
		} else if (stepName === "1") {
			assert(
				!stats.includes("<t> rebuild chunk graph"),
				"editing a dependency of an unreached module must reuse the chunk graph"
			);
			assert(
				!stats.includes("module topology change detected"),
				"unchanged unreached modules must pass the topology guard"
			);
		}
		return true;
	}
};
