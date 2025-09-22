const path = require("path");
const fs = require("fs");

const allModules = fs
	.readdirSync(__dirname, { recursive: true, withFileTypes: true })
	.filter(
		dirent =>
			dirent.isFile() &&
			dirent.name !== "package.json" &&
			dirent.name !== "rspack.config.js" &&
			dirent.name !== "test.filter.js"
	)
	.map(dirent => path.resolve(dirent.parentPath ?? dirent.path, dirent.name));

const lazyModules = new Set(
	[
		"named-barrel/b.js",
		"mixed-barrel/a.js",
		"mixed-barrel/b.js",
		"star-barrel/c.js",
		"nested-barrel/c.js"
	].map(filename => path.resolve(__dirname, filename))
);

module.exports = /** @type {import("@rspack/core").Configuration} */ ({
	experiments: {
		lazyBarrel: true
	},
	plugins: [
		function (compiler) {
			const createdModules = new Set();
			compiler.hooks.thisCompilation.tap(
				"Test",
				(compilation, { normalModuleFactory }) => {
					normalModuleFactory.hooks.createModule.tap("Test", data => {
						createdModules.add(data.resourceResolveData.resource);
					});
				}
			);
			compiler.hooks.done.tap("Test", () => {
				lazyModules.forEach(module => {
					expect(createdModules.has(module)).toBe(false);
				});
				expect(
					allModules.filter(
						module => !createdModules.has(module) && !lazyModules.has(module)
					).length
				).toBe(0);
			});
		}
	]
});
