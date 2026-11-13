import path from "node:path";
import { describeByWalk, createConfigCase } from "@rspack/test-tools";

const rspackRuntimeModeOptions = {
	experiments: {
		runtimeMode: "rspack"
	}
};
globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK = true;

// Part 2: Test cases starting with e-o
describeByWalk(
	import.meta.filename,
	(name, src, dist) => {
		createConfigCase(name, src, dist, rspackRuntimeModeOptions);
	},
	{
		source: path.join(import.meta.dirname, "configCases"),
		dist: path.resolve(import.meta.dirname, "./js/runtime-mode-config"),
		exclude: [
			// Exclude a-d
			/^[a-d]/,
			// Exclude p-z and non-ascii
			/^[p-z]/,
			/^[^a-o]/,
		]
	}
);
