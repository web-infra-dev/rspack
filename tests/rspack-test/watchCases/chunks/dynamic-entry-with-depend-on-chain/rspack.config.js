const fs = require("fs");
const path = require("path");

let ERROR;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry() {
		if (fs.existsSync(ERROR)) {
			return {
				"main-app": "./main-app.js",
				"pages/_error": {
					import: "./_error.js",
					dependOn: "pages/_app"
				},
				"pages/_app": {
					import: "./_app.js",
					dependOn: "main-app"
				}
			};
		}
		return {
			"main-app": "./main-app.js"
		};
	},
	output: {
		filename: "[name].js"
	},
	plugins: [
		{
			apply(compiler) {
				ERROR = path.join(compiler.context, "_error.js");

				compiler.hooks.finishMake.tap("PLUGIN", compilation => {
					if (!fs.existsSync(ERROR)) {
						compilation.missingDependencies.add(ERROR);
					}
				});
			}
		}
	]
};
