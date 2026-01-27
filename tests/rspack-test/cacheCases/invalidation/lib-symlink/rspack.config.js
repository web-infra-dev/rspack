const path = require("path");
const fs = require("fs");
const fsPromise = require("fs/promises");

let index = 1;
const nodeModulesPath = path.join(__dirname, "./node_modules");
const toolsV1 = path.join(__dirname, "libs/tools_v1");
const toolsV2 = path.join(__dirname, "libs/tools_v2");
const libLinkedPath = path.join(nodeModulesPath, "tools");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	experiments: {
		cache: {
			type: "persistent",
			snapshot: {
				immutablePaths: [path.join(__dirname, "./file.js")],
				managedPaths: [path.join(__dirname, "./libs"), /node_modules/]
			}
		}
	},
	plugins: [
		{
			apply(compiler) {
				// make sure the node_modules dir exist
				fs.mkdirSync(nodeModulesPath, { recursive: true });
				compiler.hooks.beforeCompile.tapPromise(
					"Test Plugin",
					async function () {
						if (index === 1) {
							await fsPromise.symlink(toolsV1, libLinkedPath);
						} else {
							await fsPromise.unlink(libLinkedPath);
							await fsPromise.symlink(toolsV2, libLinkedPath);
						}
						index++;
					}
				);
			}
		}
	]
};
