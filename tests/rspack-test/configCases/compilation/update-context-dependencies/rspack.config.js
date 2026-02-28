const path = require("path");

const PLUGIN_NAME = "plugin";
const TEST_DIR = path.resolve(__dirname, "./src/");

class Plugin {
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		compiler.hooks.afterCompile.tap(PLUGIN_NAME, compilation => {
			compilation.contextDependencies.add(TEST_DIR);
			expect(compilation.contextDependencies.has(TEST_DIR)).toBeTruthy();
			expect(
				[...compilation.contextDependencies].includes(TEST_DIR)
			).toBeTruthy();
		});
	}
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	entry: "./index.js",
	plugins: [new Plugin()]
};
