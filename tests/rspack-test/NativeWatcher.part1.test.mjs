import path from "node:path";
import { describeByWalk, createNativeWatcher } from "@rspack/test-tools";
const tempDir = path.resolve(import.meta.dirname, `./js/temp`);

// Part 1: Test cases starting with a-co (12 dirs, 37.5%)
describeByWalk(
	import.meta.filename,
	(name, src, dist) => {
		createNativeWatcher(name, src, dist, path.join(tempDir, name));
	},
	{
		source: path.join(import.meta.dirname, `./watchCases`),
		dist: path.resolve(import.meta.dirname, `./js/native-watcher/watch`),
		exclude: [
			// Exclude cp-z
			/^c[p-z]/,
			/^[d-z]/,
			// Exclude flaky tests
			/skip-building-chunk-graph/
		]
	}
);
