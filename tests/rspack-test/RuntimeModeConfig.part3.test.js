const path = require("path");
const { describeByWalk, createConfigCase } = require("@rspack/test-tools");

const rspackRuntimeModeOptions = {
	experiments: {
		runtimeMode: "rspack"
	}
};
globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK = true;

// Part 3: Test cases starting with p-z and others
describeByWalk(
	__filename,
	(name, src, dist) => {
		createConfigCase(name, src, dist, rspackRuntimeModeOptions);
	},
	{
		source: path.join(__dirname, "configCases"),
		dist: path.resolve(__dirname, "./js/runtime-mode-config"),
		exclude: [
			// Exclude a-o
			/^[a-o]/
		]
	}
);
