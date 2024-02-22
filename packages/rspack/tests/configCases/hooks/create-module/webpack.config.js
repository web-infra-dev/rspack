const fs = require("fs");
const path = require("path");

/** @type {import('@rspack/core').Configuration} */
module.exports = {
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.normalModuleFactory.tap("mock-plugin", nmf => {
					compiler.hooks.compilation.tap("mock-plugin", compilation => {
						nmf.hooks.createModule.tap("mock-plugin", createData => {
							if (createData.matchResource.includes("a.js")) {
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
