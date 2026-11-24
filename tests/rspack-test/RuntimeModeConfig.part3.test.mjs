import path from "node:path";
import { describeByWalk, createConfigCase } from "@rspack/test-tools";

const rspackRuntimeModeOptions = {
	experiments: {
		runtimeMode: "rspack"
	}
};
globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK = true;

// Part 3: Test cases starting with p-z and others
describeByWalk(
	import.meta.filename,
	(name, src, dist) => {
		createConfigCase(name, src, dist, rspackRuntimeModeOptions);
	},
	{
		source: path.join(import.meta.dirname, "configCases"),
		dist: path.resolve(import.meta.dirname, "./js/runtime-mode-config"),
		exclude: [
			// Exclude a-o
			/^[a-o]/,
		]
	}
);
