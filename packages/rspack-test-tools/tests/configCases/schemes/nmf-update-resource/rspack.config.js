/** @type {import('webpack').Configuration} */
module.exports = {
	entry: 'data:text/javascript,import "./index.js";',
	plugins: [
		function (compiler) {
			compiler.hooks.compilation.tap(
				"test",
				(compilation, { normalModuleFactory }) => {
					normalModuleFactory.hooks.afterResolve.tap("test", () => {});
				}
			);
		}
	]
};
