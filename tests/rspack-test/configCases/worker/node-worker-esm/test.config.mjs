import fs from "node:fs";
import path from "node:path";

export default {
	findBundle() {
		return "./bundle.mjs";
	},
	moduleScope(scope) {
		scope.URL = URL;
	},
	afterExecute(options) {
		const workerCode = fs.readFileSync(
			path.resolve(options.output.path, "./worker_js.bundle.mjs"),
			"utf8"
		);

		if (!/import\(\) chunk loading for javascript/.test(workerCode)) {
			throw new Error(
				"import() was not found in the worker code for loading async chunks"
			);
		}
	}
};
