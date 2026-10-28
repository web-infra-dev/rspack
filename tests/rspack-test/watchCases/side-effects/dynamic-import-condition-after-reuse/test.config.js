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
		const topologyChanged = stringStats.includes("module topology change detected");

		if (step === "0") {
			assert(rebuilt, "the cold build should build the chunk graph");
			assert(hasFeatureChunk, "the unconditional import should emit its chunk");
		} else if (step === "1") {
			assert(!topologyChanged, "the import connection should stay active");
			assert(!rebuilt, "adding an active condition should reuse the chunk graph");
			assert(hasFeatureChunk, "the used loader should keep its chunk");
		} else if (step === "2") {
			assert(topologyChanged, "the newly conditional module should be revalidated");
			assert(rebuilt, "deactivating the import should rebuild the chunk graph");
			assert(!hasFeatureChunk, "the unused loader's chunk should be removed");
		} else {
			throw new Error(`Unexpected watch step: ${step}`);
		}

		return true;
	}
};
