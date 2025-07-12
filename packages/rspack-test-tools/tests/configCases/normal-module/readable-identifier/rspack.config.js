const path = require("path");

class Plugin {
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		compiler.hooks.finishMake.tap("PLUGIN", compilation => {
			for (const module of compilation.modules) {
				if (module.resource === path.join(__dirname, "bar.js")) {
					expect(module.readableIdentifier()).toBe("./bar.js");
				}
				if (
					module.resource === path.join(__dirname, "node_modules/foo/index.js")
				) {
					expect(module.readableIdentifier()).toBe(
						"./node_modules/foo/index.js"
					);
				}
			}
		});
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	plugins: [new Plugin()]
};
