const path = require("path");
const { describeByWalk, createConfigCase } = require("@rspack/test-tools");

const rspackRuntimeModeOptions = {
	experiments: {
		runtimeMode: "rspack"
	}
};
globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK = true;

// Part 2: Test cases starting with e-o
describeByWalk(
	__filename,
	(name, src, dist) => {
		createConfigCase(name, src, dist, rspackRuntimeModeOptions);
	},
	{
		source: path.join(__dirname, "configCases"),
		dist: path.resolve(__dirname, "./js/runtime-mode-config"),
		exclude: [
			// Exclude a-d
			/^[a-d]/,
			// Exclude p-z and non-ascii
			/^[p-z]/,
			/^[^a-o]/
		]
	}
);
