const path = require("path");
const { describeByWalk, createConfigCase } = require("@rspack/test-tools");

const rspackRuntimeModeOptions = {
	experiments: {
		runtimeMode: "rspack"
	}
};
globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK = true;

// Part 1: Test cases starting with a-d
describeByWalk(
	__filename,
	(name, src, dist) => {
		createConfigCase(name, src, dist, rspackRuntimeModeOptions);
	},
	{
		source: path.join(__dirname, "configCases"),
		dist: path.resolve(__dirname, "./js/runtime-mode-config"),
		exclude: [
			// Exclude e-z and non-ascii
			/^[e-z]/,
			/^[^a-d]/,
			// Custom runtime sources are not supported in rspack runtime mode.
			/^builtin-swc-loader\/preact-refresh$/,
			/^container-1-5\/tree-shaking-shared-(infer|server)-mode$/,
			/^hooks\/(modify-extract-css-loading-runtime|rspack-issue-5571|runtime-module|runtime-requirement-in-tree)$/,
			/^rstest\/(dynamic-import-origin|mock|mock-dynamic-import-external|module-path-names|new-url-wasm)$/,
			/^runtime\/add-runtime-module/,
			/^sharing\/tree-shaking-shared$/
		]
	}
);
