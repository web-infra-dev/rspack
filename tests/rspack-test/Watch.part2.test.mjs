import path from "node:path";
import { describeByWalk, createWatchCase } from "@rspack/test-tools";
const tempDir = path.resolve(import.meta.dirname, `./js/temp`);

// Part 2: Test cases starting with cp-p (9 dirs, 28.1%)
describeByWalk(
	import.meta.filename,
	(name, src, dist) => {
		createWatchCase(name, src, dist, path.join(tempDir, name));
	},
	{
		source: path.join(import.meta.dirname, "watchCases"),
		dist: path.resolve(import.meta.dirname, `./js/watch`),
		exclude: [
			// Exclude a-co
			/^[a-c][a-o]/,
			/^[ab]/,
			// Exclude r-z
			/^[r-z]/
		]
	}
);
