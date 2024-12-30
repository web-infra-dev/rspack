const path = require("path");
const fs = require("fs/promises");

let content = 1;

const buildDependency = path.join(__dirname, "test.log");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	experiments: {
		cache: {
			type: "persistent",
			buildDependencies: [buildDependency],
			snapshot: {
				immutablePaths: [path.join(__dirname, "./file.js")]
			}
		}
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.beforeCompile.tapPromise(
					"Test Plugin",
					async function () {
						await fs.writeFile(buildDependency, String(content));
						content++;
					}
				);
			}
		}
	]
};
