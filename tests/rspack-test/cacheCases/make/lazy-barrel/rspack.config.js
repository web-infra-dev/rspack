const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	optimization: {
		// avoid analyze side effects that will change index.js dependencies at HMR
		sideEffects: "flag"
	},
	experiments: {
		cache: {
			type: "persistent"
		},
		lazyBarrel: true
	},
	plugins: [
		function (compiler) {
			let createdModules = new Set();
			compiler.hooks.compilation.tap(
				"test",
				(compilation, { normalModuleFactory }) => {
					normalModuleFactory.hooks.createModule.tap("test", data => {
						createdModules.add(data.resourceResolveData.resource);
					});
				}
			);
			compiler.hooks.done.tap("Test", () => {
				expect(createdModules.has(path.resolve(__dirname, "lib/c.js"))).toBe(
					false
				);
			});
		}
	]
};
