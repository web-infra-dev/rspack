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
		const hasFeature = source.includes("EAGER_CONDITIONAL_FEATURE_MARKER");
		const rebuilt = stringStats.includes("<t> rebuild chunk graph");
		const topologyChanged = stringStats.includes("module topology change detected");

		if (step === "0") {
			assert(rebuilt, "the cold build should build the chunk graph");
			assert(hasFeature, "the unconditional eager import should be included");
		} else if (step === "1") {
			assert(!topologyChanged, "the eager import connection should stay active");
			assert(!rebuilt, "adding an active condition should reuse the chunk graph");
			assert(hasFeature, "the used loader should keep its eager target");
		} else if (step === "2") {
			assert(topologyChanged, "the newly conditional module should be revalidated");
			assert(rebuilt, "deactivating the eager import should rebuild the chunk graph");
			assert(!hasFeature, "the unused eager target should be removed");
		} else {
			throw new Error(`Unexpected watch step: ${step}`);
		}

		return true;
	}
};
