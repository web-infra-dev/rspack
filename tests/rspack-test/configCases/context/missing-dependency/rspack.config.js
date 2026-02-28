const path = require("node:path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.done.tap("DonePlugin", stats => {
					expect(Array.from(stats.compilation.missingDependencies)).toContain(
						path.resolve(__dirname, "./lang")
					);
				});
			}
		}
	]
};
