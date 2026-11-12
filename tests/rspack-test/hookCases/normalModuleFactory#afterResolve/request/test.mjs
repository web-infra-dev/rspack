/** @type {import("@rspack/test-tools").THookCaseConfig} */
export default {
	description: "should work with request",
	options(context) {
		return {
			context: import.meta.dirname,
			entry: "./request.js",
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
										resolveData.createData.request =
											resolveData.createData.request.replace("b.js", "c.js");
										resolveData.createData.userRequest =
											resolveData.createData.userRequest.replace(
												"b.js",
												"c.js"
											);
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
