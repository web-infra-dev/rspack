const path = require("path");
const fs = require("fs/promises");

let index = 1
const libPath = path.join(__dirname, "./tool")
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	experiments: {
		cache: {
			type: "persistent",
			snapshot: {
				managedPaths: [path.join(__dirname, "./tool")]
			}
		}
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.beforeCompile.tapPromise("Test Plugin", async function () {
					try {
						await fs.unlink(libPath);
					} catch {}

					await fs.symlink(libPath + '_' + index, libPath);

					index++;
				});
			}
		}
	]
};
