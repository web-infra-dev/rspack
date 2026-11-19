import { runNodeCase } from "@rspack/test-tools/helper/node-case";

export default ["webpack", "rspack"].flatMap(runtimeMode =>
	["require", "async-node"].flatMap(chunkLoading =>
		["acceptor-replaced", "dependency-removed", "acceptor-removed"].flatMap(scenario =>
			["resolve", "reject", "pending"].map(outcome => ({
				name: `${runtimeMode}-${chunkLoading}-${scenario}-${outcome}`,
				description: "should discard pending dependency refreshes of disposed acceptors",
				async run() {
					await runNodeCase(new URL("./lifetime.mjs", import.meta.url), [
						runtimeMode,
						chunkLoading,
						scenario,
						outcome
					]);
				}
			}))
		)
	)
);
