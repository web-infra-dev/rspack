const path = require("path");
const fs = require("fs/promises");

let index = 0;
let content = 0;

const buildDependency = path.join(__dirname, "other.config.js");
const libFileA = path.join(__dirname, "node_modules/lib/src/a.js");
const libFileB = path.join(__dirname, "node_modules/lib/src/b.js");
const libFileIndex = path.join(__dirname, "node_modules/lib/src/index.js");
const libPackageJson = path.join(__dirname, "node_modules/lib/package.json");
const projectDep = path.join(__dirname, "dep.js");
const projectPackageJson = path.join(__dirname, "package.json");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	cache: {
		type: "persistent",
		buildDependencies: [buildDependency],
		snapshot: {
			immutablePaths: [path.join(__dirname, "./file.js")],
			unmanagedPaths: [path.join(__dirname, "node_modules/lib")]
		}
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.done.tapPromise("Test Plugin", async function () {
					if (index === 0) {
						await fs.writeFile(projectDep, String(content));
					} else if (index === 1) {
						await fs.writeFile(libFileA, String(content));
					} else if (index === 2) {
						await fs.writeFile(libFileB, String(content));
					} else if (index === 3) {
						await fs.writeFile(libFileIndex, String(content));
					} else if (index === 4) {
						const content = await fs.readFile(libPackageJson);
						await fs.writeFile(
							libPackageJson,
							content.toString().replace("0.0.1", "0.0.2")
						);
					} else if (index === 5) {
						const content = await fs.readFile(projectPackageJson);
						await fs.writeFile(
							projectPackageJson,
							content.toString().replace("0.0.1", "0.0.2")
						);
					}
					index++;
					content++;
				});
			}
		}
	]
};
