const path = require("path");
const { describeByWalk, createHotCase } = require("@rspack/test-tools");
const tempDir = path.resolve(__dirname, `./js/temp/runtime-mode-hot-web`);

describeByWalk(
	__filename,
	(name, src, dist) => {
		createHotCase(
			name,
			src,
			dist,
			path.join(tempDir, name),
			"web",
			{
				experiments: {
					runtimeMode: "rspack"
				}
			}
		);
	},
	{
		source: path.resolve(__dirname, "./hotCases"),
		dist: path.resolve(__dirname, `./js/runtime-mode-hot-web`)
	}
);
