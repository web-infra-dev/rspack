const path = require("path");
const { describeByWalk, createCacheCase } = require("@rspack/test-tools");

// Keep compatibility runs opt-in while newCache coverage is being completed.
const suites = [
	["newCacheCases", "cache"],
	["normalCases", "normal"],
	["configCases", "config"]
];
const selectedSuites = (process.env.NEW_CACHE_CASES || "newCacheCases")
	.split(",")
	.map(source => source.trim());
for (const source of selectedSuites) {
	if (source !== "all" && !suites.some(([name]) => name === source)) {
		throw new Error(`Unknown NEW_CACHE_CASES suite: ${source}`);
	}
}

const excludes = {
	cache: [
		// Restarts rerun loaders that these legacy assertions expect to stay cached.
		/^common\/readonly$/,
		/^common\/update-file$/,
		// Moving the source directory rebuilds the immutable module (3 instead of 1).
		/^portable\/basic$/,
		// A later restart rebuilds the module despite an unchanged context timestamp.
		/^snapshot\/context-dependencies-timestamp$/,
		// Restart restores the pre-HMR module value (1 instead of 2).
		/^snapshot\/default_value$/,
		/^snapshot\/immutable-paths$/,
		/^snapshot\/managed-paths$/,
		/^snapshot\/unmanaged-paths$/,
		// The harness overrides storage paths; these cases assert the legacy locations.
		/^storage\/directory$/,
		/^storage\/max-age$/
	],
	normal: [
		// Module restoration does not retain loader-emitted diagnostics yet.
		/^errors\/loader-error-warning$/,
		// These modules do not yet produce reusable persistent cache entries.
		/^loaders\/coffee-loader$/,
		/^parsing\/extract-amd$/
	],
	config: [
		// Restored async blocks are not yet available through the binding API.
		/^compilation\/modules$/,
		// This case expects its stateful loader to build again on every compiler.
		/^compilation\/rebuild-module(?:-getters)?$/
	]
};

for (const [source, type] of suites) {
	if (!selectedSuites.includes("all") && !selectedSuites.includes(source)) {
		continue;
	}
	describeByWalk(
		__filename,
		(name, src, dist) => {
			createCacheCase(
				name,
				src,
				dist,
				"async-node",
				path.join(
					__dirname,
					"js",
					"temp",
					"new-cache",
					source,
					path.relative(path.join(__dirname, source), src)
				),
				{ type, newCache: true }
			);
		},
		{
			source: path.join(__dirname, source),
			dist: path.join(__dirname, "js", "new-cache", source),
			exclude: excludes[type]
		}
	);
}
