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
					normalResolver.resolve(
						{},
						__dirname,
						"./index.js",
						{},
						(error, res, req) => {
							expect(error).toBeNull();
							expect(res).toBe(path.join(__dirname, "/index.js"));
							expect(req.resource).toBe(path.join(__dirname, "/index.js"));

							// With query
							normalResolver.resolve(
								{},
								__dirname,
								"./index.js?query",
								{},
								(error, res, req) => {
									expect(error).toBeNull();
									expect(res).toBe(path.join(__dirname, "/index.js?query"));
									expect(req.resource).toBe(
										path.join(__dirname, "/index.js?query")
									);
									expect(req.path).toBe(path.join(__dirname, "/index.js"));
									expect(req.query).toBe("?query");
									callback();
								}
							);
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
