const fs = require("fs");
const path = require("path");

let outputPath;

function assert(condition, message) {
	if (!condition) {
		throw new Error(`Assertion failed: ${message}`);
	}
}

module.exports = {
	findBundle(_, options) {
		outputPath = options.output.path;
		return "main.js";
	},
	checkStats(step, _, stringStats) {
		const source = fs.readFileSync(path.join(outputPath, "main.js"), "utf-8");
		const hasEagerFeature = source.includes("WATCH_EAGER_FEATURE_MARKER");
		const rebuilt = stringStats.includes("<t> rebuild chunk graph");
		const topologyChanged = stringStats.includes(
			"module topology change detected"
		);

		if (step === "0") {
			assert(rebuilt, "the cold build should build the chunk graph");
			assert(!hasEagerFeature, "the unused eager import should be omitted");
		} else if (step === "1") {
			assert(!topologyChanged, "an unchanged condition should keep the cache valid");
			assert(!rebuilt, "an unrelated edit should reuse the chunk graph");
			assert(!hasEagerFeature, "the unused eager import should stay omitted");
		} else if (step === "2") {
			assert(topologyChanged, "the active condition change should invalidate the cache");
			assert(rebuilt, "activating the eager import should rebuild the chunk graph");
			assert(hasEagerFeature, "the active eager import should be included");
		} else if (step === "3") {
			assert(topologyChanged, "the inactive condition change should invalidate the cache");
			assert(rebuilt, "deactivating the eager import should rebuild the chunk graph");
			assert(!hasEagerFeature, "the inactive eager import should be removed");
		} else {
			throw new Error(`Unexpected watch step: ${step}`);
		}

		return true;
	},
};
