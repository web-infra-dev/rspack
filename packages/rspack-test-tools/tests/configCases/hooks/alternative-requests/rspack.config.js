const pluginName = "plugin";

class Plugin {
	apply(compiler) {
		compiler.hooks.contextModuleFactory.tap(
			pluginName,
			contextModuleFactory => {
				contextModuleFactory.hooks.alternativeRequests.tap(pluginName, (requests) => {
					return requests.filter(({ request }) => request !== './b.js');
				});
			}
		);
	}
}
/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	context: __dirname,
	entry: "./index.js",
	module: {
		rules: []
	},
	plugins: [new Plugin()]
};
