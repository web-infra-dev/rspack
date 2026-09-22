import path from "node:path";
import {
	describeByWalk,
	createHotIncrementalCase
} from "@rspack/test-tools";
const tempDir = path.resolve(import.meta.dirname, `./js/temp/incremental-node`);

function v(name) {
	return path.join(import.meta.dirname, `incremental ${name}`);
}

// Run tests rspack-test/tests/hotCases in target async-node
describeByWalk(
	v("hot node"),
	(name, src, dist) => {
		createHotIncrementalCase(name, src, dist, path.join(tempDir, name), "node");
	},
	{
		source: path.resolve(import.meta.dirname, "./hotCases"),
		dist: path.resolve(import.meta.dirname, `./js/incremental/hot-node`),
		exclude: [
			/^css$/,
		]
	}
);
