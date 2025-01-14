const path = require("path");
const fs = require("fs/promises");

const cacheDir = path.join(__dirname, "node_modules/.cache/test/");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	experiments: {
		cache: {
			type: "persistent",
			storage: {
				type: "filesystem",
				directory: cacheDir
			}
		}
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.done.tapPromise("Test Plugin", async function () {
					const stat = await fs.stat(cacheDir);
					expect(stat.isDirectory()).toBeTruthy();
				});
			}
		}
	]
};
