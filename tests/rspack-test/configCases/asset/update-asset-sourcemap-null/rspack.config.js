const assert = require("assert");
const fs = require("fs");

/**
 * @type {import('@rspack/core').Configuration}
 */
module.exports = {
	context: __dirname,
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.compilation.tap("PLUGIN", compilation => {
					compilation.hooks.processAssets.tap("PLUGIN", assets => {
						for (const name in assets) {
							const source = assets[name];
							compilation.updateAsset(name, source, {
								related: { sourceMap: null }
							});
						}
					});
				});
			}
		}
	]
};
