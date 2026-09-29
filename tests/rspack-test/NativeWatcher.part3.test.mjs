import path from "node:path";
import { describeByWalk, createNativeWatcher } from "@rspack/test-tools";
const tempDir = path.resolve(import.meta.dirname, `./js/temp`);

// Part 3: Test cases starting with r-z (11 dirs, 34.4%)
describeByWalk(
	import.meta.filename,
	(name, src, dist) => {
		createNativeWatcher(name, src, dist, path.join(tempDir, name));
	},
	{
		source: path.join(import.meta.dirname, `./watchCases`),
		dist: path.resolve(import.meta.dirname, `./js/native-watcher/watch`),
		exclude: [
			// Exclude a-p
			/^[a-p]/,
			// Exclude flaky tests
			/skip-building-chunk-graph/
		]
	}
);
