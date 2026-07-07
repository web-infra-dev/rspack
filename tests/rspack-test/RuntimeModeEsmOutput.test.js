const path = require("path");
const { describeByWalk, createEsmOutputCase } = require("@rspack/test-tools");

const rspackRuntimeModeOptions = {
	experiments: {
		runtimeMode: "rspack"
	}
};
globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK = true;

describeByWalk(
	__filename,
	(name, src, dist) => {
		createEsmOutputCase(name, src, dist, rspackRuntimeModeOptions);
	},
	{
		source: path.resolve(__dirname, "./esmOutputCases"),
		dist: path.resolve(__dirname, "./js/runtime-mode-esm-output"),
		exclude: [
			// ESM chunk loading in rspack runtime mode currently expects
			// chunk-format metadata that these context import cases do not emit.
			/^dynamic-import\/import-context-(lazy|multi-chunk|prefetch-preload)$/
		]
	}
);
