import { fileURLToPath } from "node:url";

const tinypoolUrl = new URL(
	"compiled/tinypool/dist/index.js",
	import.meta.resolve("@rspack/core/package.json")
);

export default {
	moduleScope(scope) {
		// Import the published copy outside the test runner's VM.
		scope.loadTinypool = () => import(tinypoolUrl.href);
		scope.workerFilename = fileURLToPath(new URL("./worker.mjs", import.meta.url));
	}
};
