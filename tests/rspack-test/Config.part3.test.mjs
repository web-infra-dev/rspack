import path from "node:path";
import { describeByWalk, createConfigCase } from "@rspack/test-tools";

// Part 3: Test cases starting with p-z and others (44 dirs, 32.4%)
describeByWalk(
	import.meta.filename,
	(name, src, dist) => {
		createConfigCase(name, src, dist);
	},
	{
		source: path.join(import.meta.dirname, "configCases"),
		dist: path.resolve(import.meta.dirname, `./js/config`),
		exclude: [
			// Exclude a-o
			/^[a-o]/
		]
	}
);
