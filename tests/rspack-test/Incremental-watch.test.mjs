import path from "node:path";
import {
	describeByWalk,
	createWatchIncrementalCase
} from "@rspack/test-tools";

process.env.RSPACK_INCREMENTAL_WATCH_TEST = true;

function v(name) {
	return path.join(import.meta.dirname, `incremental ${name}`);
}

// Run tests rspack-test/tests/watchCases
describeByWalk(
	v("watch"),
	(name, src, dist) => {
		const tempDir = path.resolve(import.meta.dirname, `./js/incremental/temp`);
		createWatchIncrementalCase(name, src, dist, path.join(tempDir, name));
	},
	{
		source: path.resolve(import.meta.dirname, "./watchCases"),
		dist: path.resolve(import.meta.dirname, `./js/incremental/watch`)
	}
);
