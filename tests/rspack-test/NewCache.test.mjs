import path from "node:path";
import { describeByWalk, createCacheCase } from "@rspack/test-tools";

const source = path.join(import.meta.dirname, "newCacheCases");

describeByWalk(
	import.meta.filename,
	(name, src, dist) => {
		const temp = path.join(
			import.meta.dirname,
			"js",
			"temp",
			"new-cache",
			"newCacheCases",
			path.relative(source, src)
		);
		createCacheCase(name, src, dist, "async-node", temp, { newCache: true });
	},
	{
		source,
		dist: path.join(import.meta.dirname, "js", "new-cache", "newCacheCases"),
		exclude: [
			// Expects legacy storage metadata; newCache does not implement maxAge yet.
			/^storage\/max-age$/
		]
	}
);
