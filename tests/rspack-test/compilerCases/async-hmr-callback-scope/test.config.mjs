import { runNodeCase } from "@rspack/test-tools/helper/node-case";

export default ["webpack", "rspack"].flatMap(runtimeMode =>
	[false, true].flatMap(asyncDependency =>
		["arguments", "super", "new-target", "fallback"].map(scenario => ({
			name: `${runtimeMode}-${asyncDependency ? "async" : "sync"}-${scenario}`,
			description: "should preserve lexical bindings of accept callbacks",
			async run() {
				await runNodeCase(new URL("./scope.mjs", import.meta.url), [
					runtimeMode,
					asyncDependency,
					scenario
				]);
			}
		}))
	)
);
