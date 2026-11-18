function assert(condition, message) {
	if (!condition) {
		throw new Error(`Assertion failed: ${message}`);
	}
}

module.exports = {
	findBundle() {
		return "main.js";
	},
	checkStats(step, stats, stringStats) {
		const hasFeatureChunk = stats.assets.some(asset => asset.name === "feature.js");
		const rebuilt = stringStats.includes("<t> rebuild chunk graph");
		const topologyChanged = stringStats.includes(
			"module topology change detected"
		);

		if (step === "0") {
			assert(rebuilt, "the cold build should build the chunk graph");
			assert(!hasFeatureChunk, "the unused dynamic import should not emit a chunk");
		} else if (step === "1") {
			assert(!topologyChanged, "an unchanged condition should keep the cache valid");
			assert(!rebuilt, "an unrelated edit should reuse the chunk graph");
			assert(!hasFeatureChunk, "the unused dynamic import should stay omitted");
		} else if (step === "2") {
			assert(rebuilt, "activating the dynamic import should rebuild the chunk graph");
			assert(topologyChanged, "the active condition change should invalidate the cache");
			assert(hasFeatureChunk, "the active dynamic import should emit its chunk");
		} else if (step === "3") {
			assert(rebuilt, "deactivating the dynamic import should rebuild the chunk graph");
			assert(topologyChanged, "the inactive condition change should invalidate the cache");
			assert(!hasFeatureChunk, "the inactive dynamic import chunk should be removed");
		} else {
			throw new Error(`Unexpected watch step: ${step}`);
		}

		return true;
	}
};
