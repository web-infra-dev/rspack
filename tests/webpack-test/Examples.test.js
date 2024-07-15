"use strict";

// require("./helpers/warmup-webpack");

const path = require("path");
const fs = require("graceful-fs");
const { normalizeFilteredTestName, FilteredStatus } = require("./lib/util/filterUtil")

describe("Examples", () => {
	const basePath = path.join(__dirname, "..", "webpack-examples");
	const examples = require("../webpack-examples/examples.js");

	examples.forEach(examplePath => {
		const filterPath = path.join(examplePath, "test.filter.js");
		const relativePath = path.relative(basePath, examplePath);
		if (fs.existsSync(filterPath)) {
			let flag = require(filterPath)()
			let shouldRun = flag === true || (Array.isArray(flag) && flag.includes(FilteredStatus.PARTIAL_PASS))
			if (!shouldRun) {
				let filteredName = normalizeFilteredTestName(flag, relativePath);
				describe.skip(relativePath, () => {
					it(filteredName, () => { });
				});
				return;
			}
		}
		it(
			"should compile " + relativePath,
			function (done) {
				let options = {};
				let webpackConfigPath = path.join(examplePath, "webpack.config.js");
				webpackConfigPath =
					webpackConfigPath.slice(0, 1).toUpperCase() +
					webpackConfigPath.slice(1);
				if (fs.existsSync(webpackConfigPath))
					options = require(webpackConfigPath);
				if (typeof options === "function") options = options();
				if (Array.isArray(options)) options.forEach(processOptions);
				else processOptions(options);

				function processOptions(options) {
					options.context = examplePath;
					options.output = options.output || {};
					options.output.pathinfo = true;
					options.output.path = path.join(examplePath, "dist");
					options.output.publicPath = "dist/";
					if (!options.entry) options.entry = "./example.js";
					if (!options.plugins) options.plugins = [];
				}
				const webpack = require("@rspack/core").rspack;
				webpack(options, (err, stats) => {
					if (err) return done(err);
					if (stats.hasErrors()) {
						return done(
							new Error(
								stats.toString({
									all: false,
									errors: true,
									errorDetails: true,
									errorStacks: true
								})
							)
						);
					}
					done();
				});
			},
			90000
		);
	});
});
