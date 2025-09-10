/** @type {import('@rspack/core').Configuration} */
module.exports = {
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.normalModuleFactory.tap("mock-plugin", nmf => {
					compiler.hooks.compilation.tap("mock-plugin", compilation => {
						nmf.hooks.createModule.tap("mock-plugin", createData => {
							if (createData.matchResource?.endsWith(".vanilla.css")) {
								const { RawSource } = compiler.webpack.sources;
								compilation.emitAsset(
									"./createData.json",
									new RawSource(JSON.stringify(createData, null, 2)),
									{}
								);
							}
						});
					});
				});
			}
		}
	]
};
