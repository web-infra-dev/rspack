import path from "node:path";
import { describeByWalk, createHotCase } from "@rspack/test-tools";
const tempDir = path.resolve(import.meta.dirname, `./js/temp/runtime-mode-hot-node`);

describeByWalk(
	import.meta.filename,
	(name, src, dist) => {
		createHotCase(
			name,
			src,
			dist,
			path.join(tempDir, name),
			"async-node",
			{
				experiments: {
					runtimeMode: "rspack"
				}
			}
		);
	},
	{
		source: path.resolve(import.meta.dirname, "./hotCases"),
		dist: path.resolve(import.meta.dirname, `./js/runtime-mode-hot-node`),
		exclude: [/^css$/]
	}
);
