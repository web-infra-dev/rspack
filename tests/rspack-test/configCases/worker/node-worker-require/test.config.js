"use strict";

const fs = require("fs");
const path = require("path");

module.exports = {
	findBundle() {
		return "./bundle.js";
	},
	afterExecute(options) {
		const bundleCode = fs.readFileSync(
			path.resolve(options.output.path, "./bundle.js"),
			"utf8"
		);
		const workerCode = fs.readFileSync(
			path.resolve(options.output.path, "./worker_js.bundle.js"),
			"utf8"
		);

		if (/var __rspack_context\s*=/.test(bundleCode)) {
			expect(bundleCode).not.toMatch(/\bmoduleCache\s*=\s*typeof __rspack_module_cache/);
			expect(bundleCode).not.toMatch(/\bmoduleFactories\s*=\s*typeof __rspack_modules/);
		}

		if (!/require\(\) chunk loading for javascript/.test(workerCode)) {
			throw new Error(
				"require was not found in the worker code for loading async chunks"
			);
		}
	}
};
