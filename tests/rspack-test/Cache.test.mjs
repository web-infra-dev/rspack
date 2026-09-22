import path from "node:path";
import { describeByWalk, createCacheCase } from "@rspack/test-tools";
const tempDir = path.resolve(import.meta.dirname, `./js/temp`);

// Run tests rspack-test/tests/cacheCases in target async-node
describeByWalk(
	import.meta.filename,
	(name, src, dist) => {
		createCacheCase(name, src, dist, "async-node", path.join(tempDir, name));
	},
	{
		source: path.resolve(import.meta.dirname, "./cacheCases"),
		dist: path.resolve(import.meta.dirname, `./js/cache/async-node`),
		exclude: [/^css$/]
	}
);
