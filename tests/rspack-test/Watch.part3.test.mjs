import path from "node:path";
import { describeByWalk, createWatchCase } from "@rspack/test-tools";
const tempDir = path.resolve(import.meta.dirname, `./js/temp`);

// Part 3: Test cases starting with r-z (11 dirs, 34.4%)
describeByWalk(
	import.meta.filename,
	(name, src, dist) => {
		createWatchCase(name, src, dist, path.join(tempDir, name));
	},
	{
		source: path.join(import.meta.dirname, "watchCases"),
		dist: path.resolve(import.meta.dirname, `./js/watch`),
		exclude: [
			// Exclude a-p
			/^[a-p]/
		]
	}
);
