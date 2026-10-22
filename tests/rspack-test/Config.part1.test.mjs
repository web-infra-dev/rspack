import path from "node:path";
import { describeByWalk, createConfigCase } from "@rspack/test-tools";

// Part 1: Test cases starting with a-d (49 dirs, 36.0%)
describeByWalk(
	import.meta.filename,
	(name, src, dist) => {
		createConfigCase(name, src, dist);
	},
	{
		source: path.join(import.meta.dirname, "configCases"),
		dist: path.resolve(import.meta.dirname, `./js/config`),
		exclude: [
			// Exclude e-z and non-ascii
			/^[e-z]/,
			/^[^a-d]/
		]
	}
);
