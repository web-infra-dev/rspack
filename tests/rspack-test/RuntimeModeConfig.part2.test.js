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
			/^[^a-o]/,
			// Custom runtime sources are not supported in rspack runtime mode.
			/^builtin-swc-loader\/preact-refresh$/,
			/^container-1-5\/tree-shaking-shared-(infer|server)-mode$/,
			/^hooks\/(modify-extract-css-loading-runtime|rspack-issue-5571|runtime-module|runtime-requirement-in-tree)$/,
			/^rstest\/(dynamic-import-origin|mock|mock-dynamic-import-external|mock-shared-runtime|module-path-names|new-url-wasm)$/,
			/^runtime\/add-runtime-module/,
			/^sharing\/tree-shaking-shared$/
		]
	}
);
