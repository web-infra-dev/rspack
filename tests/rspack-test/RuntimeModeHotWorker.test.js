const path = require("path");
const { describeByWalk, createHotCase } = require("@rspack/test-tools");
const tempDir = path.resolve(__dirname, `./js/temp/runtime-mode-hot-worker`);

describeByWalk(
	__filename,
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
		source: path.resolve(__dirname, "./hotCases"),
		dist: path.resolve(__dirname, `./js/runtime-mode-hot-worker`),
		exclude: [/^css$/]
	}
);
