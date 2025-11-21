const path = require("path");
const file = path.join(__dirname, "a.js");
const asyncLoader = path.join(__dirname, "asyncloader.js");
const syncLoader = path.join(__dirname, "syncloader.js");

/**
 * Tests ported from webpack https://github.com/webpack/webpack/blob/8241da7f1e75c5581ba535d127fa66aeb9eb2ac8/test/cases/loaders/async/index.js
 *
 * As the fact that we currently don't support inline-loader-syntax, so we define these tests with `resourceQuery`
 */

const createRule = (testNumber, loaders) => ({
	test: file,
	resourceQuery: new RegExp("case-" + testNumber),
	use: loaders.map(loader => ({
		loader
	}))
});

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			createRule(1, [syncLoader]),
			createRule(2, [asyncLoader]),
			createRule(3, [syncLoader, syncLoader]),
			createRule(4, [syncLoader, asyncLoader]),
			createRule(5, [asyncLoader, syncLoader]),
			createRule(6, [asyncLoader, asyncLoader]),
			createRule(7, [asyncLoader, asyncLoader, asyncLoader]),
			createRule(8, [asyncLoader, syncLoader, asyncLoader]),
			createRule(9, [syncLoader, asyncLoader, syncLoader]),
			createRule(10, [syncLoader, syncLoader, syncLoader])
		]
	}
};
