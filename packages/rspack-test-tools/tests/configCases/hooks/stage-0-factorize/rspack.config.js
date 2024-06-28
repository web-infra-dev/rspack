const TestPlugin = require("../stage-compilation/plugin");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	externals: ['commonjs fs', 'commonjs path'],
	plugins: [new TestPlugin((compiler, list) => {
		compiler.hooks.compilation.tap(TestPlugin.name, (compilation, { normalModuleFactory }) => {
			normalModuleFactory.hooks.factorize.tap(TestPlugin.name, (resolveData) => {
				list.push(`/* ${resolveData.request} */`);
			})
		});
	})]
};
