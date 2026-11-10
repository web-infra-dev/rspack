import { runNodeCase } from "@rspack/test-tools/helper/node-case";

export default ["webpack", "rspack"].flatMap(runtimeMode =>
	["require", "async-node"].flatMap(chunkLoading =>
		[
			"accept-error",
			"chunk-version",
			"chunk-version-manual",
			"chunk-version-same-dependency",
			"chunk-version-late",
			"chunk-version-missing-manifest",
			"chunk-version-pending",
			"chunk-version-multiple",
			"chunk-version-error",
			"chunk-version-status-error"
		].map(scenario => ({
			name: `${runtimeMode}-${chunkLoading}-${scenario}`,
			description:
				"should recover from accepted errors and newer imported chunks",
			async run() {
				await runNodeCase(new URL("./recovery.mjs", import.meta.url), [
					runtimeMode,
					chunkLoading,
					scenario
				]);
			}
		}))
	)
);
