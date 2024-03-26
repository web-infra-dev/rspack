const pluginName = "plugin";

/**@type {import('@rspack/cli').Configuration}*/
module.exports = [
	{
		context: __dirname,
		entry: "./resource.js",
		output: {
			pathinfo: false,
			filename: "resource.js"
		},
		optimization: {
			moduleIds: "named"
		},
		plugins: [{
			apply(compiler) {
				compiler.hooks.compilation.tap(
					pluginName,
					(compilation, { normalModuleFactory }) => {
						normalModuleFactory.hooks.afterResolve.tap(pluginName, resolveData => {
							resolveData.createData.resource = resolveData.createData.resource.replace("b.js", "c.js");
						});
					}
				);
			}
		}]
	},
	{
		context: __dirname,
		entry: "./request.js",
		output: {
			pathinfo: false,
			filename: "request.js"
		},
		optimization: {
			moduleIds: "named"
		},
		plugins: [{
			apply(compiler) {
				compiler.hooks.compilation.tap(
					pluginName,
					(compilation, { normalModuleFactory }) => {
						normalModuleFactory.hooks.afterResolve.tap(pluginName, resolveData => {
							resolveData.createData.request = resolveData.createData.request.replace("b.js", "c.js");
							resolveData.createData.userRequest = resolveData.createData.userRequest.replace("b.js", "c.js");
						});
					}
				);
			}
		}]
	},
	{
		context: __dirname,
		entry: "./duplicate.js",
		output: {
			pathinfo: false,
			filename: "duplicate.js"
		},
		optimization: {
			moduleIds: "named"
		},
		plugins: [{
			apply(compiler) {
				compiler.hooks.compilation.tap(
					pluginName,
					(compilation, { normalModuleFactory }) => {
						normalModuleFactory.hooks.afterResolve.tap(pluginName, resolveData => {
							resolveData.createData.request = resolveData.createData.request.replace("b.js", "c.js");
							resolveData.createData.userRequest = resolveData.createData.userRequest.replace("b.js", "c.js");
							resolveData.createData.resource = resolveData.createData.resource.replace("b.js", "c.js");
						});
					}
				);
			}
		}]
	},
];
