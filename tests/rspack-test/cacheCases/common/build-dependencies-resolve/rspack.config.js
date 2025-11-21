const path = require("path");
const fs = require("fs/promises");

let index = 0;
let content = 0;

const buildDependency = path.join(__dirname, "other.config.js");
const buildDependencyA = path.join(__dirname, "node_modules/lib/src/a.js");
const buildDependencyB = path.join(__dirname, "node_modules/lib/src/b.js");
const buildDependencyIndex = path.join(
	__dirname,
	"node_modules/lib/src/index.js"
);

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	experiments: {
		cache: {
			type: "persistent",
			buildDependencies: [buildDependency],
			snapshot: {
				immutablePaths: [path.join(__dirname, "./file.js")],
				unmanagedPaths: [path.join(__dirname, "node_modules/lib")]
			}
		}
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.done.tapPromise("Test Plugin", async function () {
					if (index === 1) {
						await fs.writeFile(buildDependencyA, String(content));
					} else if (index === 2) {
						await fs.writeFile(buildDependencyB, String(content));
					} else if (index === 3) {
						await fs.writeFile(buildDependencyIndex, String(content));
					}
					index++;
					content++;
				});
			}
		}
	]
};
