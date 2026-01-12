const path = require("path");

let index = 1;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	cache: {
		type: "persistent",
		snapshot: {
			immutablePaths: [path.join(__dirname, "./file.js")]
		}
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.beforeCompile.tap("Test Plugin", function () {
					if (index === 1) {
						compiler.options.mode = "development";
					} else {
						compiler.options.mode = "production";
					}
					index++;
				});
			}
		}
	]
};
