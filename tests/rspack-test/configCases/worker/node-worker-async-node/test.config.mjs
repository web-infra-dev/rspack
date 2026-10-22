import fs from "node:fs";
import path from "node:path";

export default {
	findBundle() {
		return "./bundle.js";
	},
	afterExecute(options) {
		const workerCode = fs.readFileSync(
			path.resolve(options.output.path, "./worker_js.bundle.js"),
			"utf8"
		);

		if (!/ReadFile \+ VM\.run chunk loading for javascript/.test(workerCode)) {
			throw new Error(
				"require('fs').readFile(...) and require('vm').runInThisContext were not found in the worker code for loading async chunks"
			);
		}
	}
};
