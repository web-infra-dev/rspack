const pluginName = "plugin";

class Plugin {
	apply(compiler) {
		compiler.hooks.contextModuleFactory.tap(
			pluginName,
			contextModuleFactory => {
				contextModuleFactory.hooks.afterResolve.tap(pluginName, resolveData => {
					if (resolveData.request.includes("./dir")) {
						return false;
					}
				});
			}
		);
	}
}
/**@type {import('@rspack/cli').Configuration}*/
module.exports = {
	context: __dirname,
	entry: "./index.js",
	module: {
		rules: []
	},
	plugins: [new Plugin()]
};
