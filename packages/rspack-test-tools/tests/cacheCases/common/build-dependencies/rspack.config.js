const path = require("path");
const fs = require("fs/promises");

let content = 1;

const logA = path.join(__dirname, "./configs/a.log");
const logB = path.join(__dirname, "./configs/b.log");
const BuildDependency = path.join(__dirname, "./configs/index.js");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	experiments: {
		cache: {
			type: "persistent",
			buildDependencies: [BuildDependency],
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
						if (content == 1) {
							// init
							await fs.writeFile(logA, String(content));
							await fs.writeFile(logB, String(content));
						} else if (content == 2) {
							// do nothing
						} else if (content == 3) {
							// update a.log
							await fs.writeFile(logA, String(content));
						} else if (content == 4) {
							// update b.log
							await fs.writeFile(logB, String(content));
						} else if (content == 5) {
							// do nothing
						} else if (content == 6) {
							await fs.writeFile(BuildDependency, String(content));
						}
						content++;
					}
				);
			}
		}
	]
};
