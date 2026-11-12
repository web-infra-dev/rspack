import path from "node:path";
import { describeByWalk, createConfigCase } from "@rspack/test-tools";
import { getCacheTestOptions } from "./cacheTestOptions.mjs";

// Part 2: Test cases starting with e-o (43 dirs, 31.6%)
describeByWalk(
	import.meta.filename,
	(name, src, dist) => {
		createConfigCase(
			name,
			src,
			dist,
			undefined,
			getCacheTestOptions("config", src)
		);
	},
	{
		source: path.join(import.meta.dirname, "configCases"),
		dist: path.resolve(import.meta.dirname, `./js/config`),
		exclude: [
			// Exclude a-d
			/^[a-d]/,
			// Exclude p-z and non-ascii
			/^[p-z]/,
			/^[^a-o]/
		]
	}
);
