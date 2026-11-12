import path from "node:path";
import {
	describeByWalk,
	createHotIncrementalCase
} from "@rspack/test-tools";
const tempDir = path.resolve(import.meta.dirname, `./js/temp/incremental-web`);

function v(name) {
	return path.join(import.meta.dirname, `incremental ${name}`);
}

// Run tests rspack-test/tests/hotCases in target web
describeByWalk(
	v("hot web"),
	(name, src, dist) => {
		createHotIncrementalCase(name, src, dist, path.join(tempDir, name), "web");
	},
	{
		source: path.resolve(import.meta.dirname, "./hotCases"),
		dist: path.resolve(import.meta.dirname, `./js/incremental/hot-web`)
	}
);
