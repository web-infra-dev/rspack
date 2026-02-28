const path = require("path");

class Plugin {
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		compiler.hooks.compilation.tap("PLUGIN", compilation => {
			compilation.hooks.finishModules.tapAsync(
				"PLUGIN",
				(modules, callback) => {
					const normalResolver = compiler.resolverFactory.get("normal");
					// With fragment
					normalResolver.resolve(
						{},
						__dirname,
						"./index.js#fragment",
						{},
						(error, res, req) => {
							expect(
								normalResolver.resolveSync({}, __dirname, "./index.js#fragment")
							).toBe(res);

							expect(error).toBeNull();
							expect(res).toBe(path.join(__dirname, "/index.js#fragment"));
							// Webpack does not have resource field
							expect(req.resource).toBe(undefined);
							expect(req.path).toBe(path.join(__dirname, "/index.js"));
							expect(req.fragment).toBe("#fragment");
							callback();
						}
					);
				}
			);
		});
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	plugins: [new Plugin()]
};
