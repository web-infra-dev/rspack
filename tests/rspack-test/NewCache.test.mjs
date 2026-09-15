import path from "node:path";
import { describeByWalk, createCacheCase } from "@rspack/test-tools";

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
		// Expects legacy storage metadata; newCache does not implement maxAge yet.
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
		import.meta.filename,
		(name, src, dist) => {
			createCacheCase(
				name,
				src,
				dist,
				"async-node",
				path.join(
					import.meta.dirname,
					"js",
					"temp",
					"new-cache",
					source,
					path.relative(path.join(import.meta.dirname, source), src)
				),
				{ type, newCache: true }
			);
		},
		{
			source: path.join(import.meta.dirname, source),
			dist: path.join(import.meta.dirname, "js", "new-cache", source),
			exclude: excludes[type]
		}
	);
}
