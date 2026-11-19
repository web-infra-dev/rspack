import { runNodeCase } from "@rspack/test-tools/helper/node-case";

export default ["webpack", "rspack"].flatMap(runtimeMode =>
	["require", "async-node"].flatMap(chunkLoading =>
		[
			"check",
			"prepare",
			"ready",
			"check-current",
			"prepare-current",
			"ready-current",
			"ignore-error",
			"ignore-error-manual",
			"report-error",
			"ignore-declined",
			"syntax-single",
			"syntax-multiple",
			"syntax-no-callback"
		].map(scenario => ({
			name: `${runtimeMode}-${chunkLoading}-${scenario}`,
			description:
				"should preserve async HMR recovery and code generation contracts",
			async run() {
				await runNodeCase(new URL("./boundaries.mjs", import.meta.url), [
					runtimeMode,
					chunkLoading,
					scenario
				]);
			}
		}))
	)
);
