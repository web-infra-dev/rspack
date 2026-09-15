import path from "node:path";
import { describeByWalk, createEsmOutputCase } from "@rspack/test-tools";

const rspackRuntimeModeOptions = {
	experiments: {
		runtimeMode: "rspack"
	}
};
globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK = true;

describeByWalk(
	import.meta.filename,
	(name, src, dist) => {
		createEsmOutputCase(name, src, dist, rspackRuntimeModeOptions);
	},
	{
		source: path.resolve(import.meta.dirname, "./esmOutputCases"),
		dist: path.resolve(import.meta.dirname, "./js/runtime-mode-esm-output")
	}
);
