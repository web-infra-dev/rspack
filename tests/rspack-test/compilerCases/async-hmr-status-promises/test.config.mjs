import { runNodeCase } from "@rspack/test-tools/helper/node-case";

export default ["webpack", "rspack"].flatMap(runtimeMode =>
	["dispose", "apply"].flatMap(status =>
		["resolve", "reject", "reject-before-status"].map(acceptOutcome => ({
			name: `${runtimeMode}-${status}-${acceptOutcome}`,
			description: "should observe status rejections while an async accept is pending",
			async run() {
				await runNodeCase(new URL("./status-promises.mjs", import.meta.url), [
					runtimeMode,
					status,
					acceptOutcome
				]);
			}
		}))
	)
);
