const path = require("path");
const fs = require("fs/promises");

let content = 1;
let index = 0;

const buildDependency = path.join(__dirname, "test");
const buildDependencyFile1 = path.join(buildDependency, "test1.log");
const buildDependencyFile2 = path.join(buildDependency, "test2", "test2.log");

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
						if (index === 0) {
							await fs.writeFile(buildDependencyFile1, String(content));
							index++;
							content++;
						} else if (index === 1) {
							await fs.writeFile(buildDependencyFile1, String(content));
							index++;
							content++;
						} else if (index === 2) {
							await fs.writeFile(buildDependencyFile2, String(content));
							index++;
							content++;
						} else if (index === 3) {
							await fs.writeFile(buildDependencyFile2, String(content));
							index++;
							content++;
						} else if (index === 4) {
							await fs.writeFile(buildDependencyFile1, String(content));
							content++;
							await fs.writeFile(buildDependencyFile2, String(content));
							index++;
							content++;
						} else if (index === 5) {
							await fs.writeFile(buildDependencyFile1, String(content));
							content++;
							await fs.writeFile(buildDependencyFile2, String(content));
							index++;
							content++;
						}
					}
				);
			}
		}
	]
};
