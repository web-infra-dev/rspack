const { createFsFromVolume, Volume } = require("memfs");

const outputFileSystem = createFsFromVolume(new Volume());

/** @type {import("../../../..").THookCaseConfig} */
module.exports = {
	description: "should work with request",
	options(context) {
		return {
			context: __dirname,
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
	async compiler(context, compiler) {
		compiler.outputFileSystem = outputFileSystem;
	},
	async check() { }
};
