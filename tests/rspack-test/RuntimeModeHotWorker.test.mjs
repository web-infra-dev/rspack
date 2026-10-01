import path from "node:path";
import { describeByWalk, createHotCase } from "@rspack/test-tools";
const tempDir = path.resolve(import.meta.dirname, `./js/temp/runtime-mode-hot-worker`);

describeByWalk(
	import.meta.filename,
	(name, src, dist) => {
		createHotCase(
			name,
			src,
			dist,
			path.join(tempDir, name),
			"webworker",
			{
				experiments: {
					runtimeMode: "rspack"
				}
			}
		);
	},
	{
		source: path.resolve(import.meta.dirname, "./hotCases"),
		dist: path.resolve(import.meta.dirname, `./js/runtime-mode-hot-worker`),
		exclude: [/^css$/]
	}
);
