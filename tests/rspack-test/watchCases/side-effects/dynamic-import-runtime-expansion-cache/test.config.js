function assert(condition, message) {
	if (!condition) {
		throw new Error(`Assertion failed: ${message}`);
	}
}

module.exports = {
	findBundle() {
		return ["a.js", "b.js"];
	},
	checkStats(step, _, stringStats) {
		const rebuilt = stringStats.includes("<t> rebuild chunk graph");
		const conditionChanged = stringStats.includes(
			"async dependency condition change detected"
		);

		if (step === "0") {
			assert(rebuilt, "the cold build should build the chunk graph");
		} else if (step === "1" || step === "2") {
			assert(rebuilt, `runtime usage change at step ${step} should rebuild`);
			assert(
				conditionChanged,
				`runtime-specific condition change at step ${step} should invalidate the cache`
			);
		} else {
			throw new Error(`Unexpected watch step: ${step}`);
		}

		return true;
	}
};
