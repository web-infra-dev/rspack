import { runNodeCase } from "@rspack/test-tools/helper/node-case";

export default ["webpack", "rspack"].map(runtimeMode => ({
	name: runtimeMode,
	description: "should collect old async exports after repeated accepted updates",
	async run() {
		await runNodeCase(new URL("./exports-gc.mjs", import.meta.url), [runtimeMode]);
	}
}));
