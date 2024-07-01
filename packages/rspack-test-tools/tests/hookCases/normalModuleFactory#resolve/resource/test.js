/** @type {import("../../../..").THookCaseConfig} */
module.exports = {
	description: "should work with resource",
	options(context) {
		return {
			context: __dirname,
			entry: "./resource.js",
			optimization: {
				moduleIds: "named"
			},
			plugins: [
				{
					apply(compiler) {
						compiler.hooks.compilation.tap(
							"plugin",
							(compilation, { normalModuleFactory }) => {
								normalModuleFactory.hooks.afterResolve.tap(
									"plugin",
									resolveData => {
										resolveData.createData.resource =
											resolveData.createData.resource.replace("b.js", "c.js");
									}
								);
							}
						);
					}
				}
			]
		};
	},
};
