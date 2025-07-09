const path = require("node:path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.done.tap("DonePlugin", stats => {
					let missingDependencies = Array.from(
						stats.compilation.missingDependencies
					).filter(item => !item.endsWith("package.json"));
					expect(missingDependencies).toEqual([
						path.resolve(__dirname, "./lang")
					]);
				});
			}
		}
	]
};
