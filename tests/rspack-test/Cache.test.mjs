import path from "node:path";
import { describeByWalk, createCacheCase } from "@rspack/test-tools";
const tempDir = path.resolve(import.meta.dirname, `./js/temp`);

// Run tests rspack-test/tests/cacheCases in target async-node
describeByWalk(
	import.meta.filename,
	(name, src, dist) => {
		createCacheCase(name, src, dist, "async-node", path.join(tempDir, name));
		if (new Set([
			"snapshot-refresh-after-compile",
			"snapshot-refresh-plugin-overlap",
			"context-snapshot-after-compile"
		]).has(path.basename(src)) && path.basename(path.dirname(src)) === "portable") {
			const variant = `${name}-non-portable`;
			createCacheCase(variant, src, `${dist}-non-portable`, "async-node", path.join(tempDir, variant), { portable: false });
		}
	},
	{
		source: path.resolve(import.meta.dirname, "./cacheCases"),
		dist: path.resolve(import.meta.dirname, `./js/cache/async-node`),
		exclude: [/^css$/]
	}
);
