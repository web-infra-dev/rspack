import { runNodeCase } from "@rspack/test-tools/helper/node-case";

export default ["webpack", "rspack"].flatMap(runtimeMode =>
	["require", "async-node"].flatMap(chunkLoading =>
		[
			"removed-self",
			"removed-self-runtime",
			"updated-ids",
			"await-sync",
			"await-async",
			"yield-sync",
			"yield-async",
			"await-noarrow",
			"yield-noarrow"
		].map(scenario => ({
			name: `${runtimeMode}-${chunkLoading}-${scenario}`,
			description:
				"should preserve nested update results and suspending accept arguments",
			async run() {
				await runNodeCase(new URL("./review.mjs", import.meta.url), [
					runtimeMode,
					chunkLoading,
					scenario
				]);
			}
		}))
	)
);
