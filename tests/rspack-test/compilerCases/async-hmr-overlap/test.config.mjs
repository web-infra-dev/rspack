import { runNodeCase } from "@rspack/test-tools/helper/node-case";

export default ["webpack", "rspack"].flatMap(runtimeMode =>
	["require", "async-node"].flatMap(chunkLoading =>
		["stale-exports", "stale-rejection", "parent-status-error"].map(scenario => ({
			name: `${runtimeMode}-${chunkLoading}-${scenario}`,
			description:
				"should preserve dependency versions and failed parents across nested updates",
			async run() {
				await runNodeCase(new URL("./overlap.mjs", import.meta.url), [
					runtimeMode,
					chunkLoading,
					scenario
				]);
			}
		}))
	)
);
