/** @type {import("../../../..").THookCaseConfig} */
module.exports = {
	description: "should work with duplicate",
	options(context) {
		return {
			context: __dirname,
			entry: "./duplicate.js",
			optimization: {
				moduleIds: "named",
				concatenateModules: false
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
										resolveData.createData.request =
											resolveData.createData.request.replace("b.js", "c.js");
										resolveData.createData.userRequest =
											resolveData.createData.userRequest.replace(
												"b.js",
												"c.js"
											);
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
