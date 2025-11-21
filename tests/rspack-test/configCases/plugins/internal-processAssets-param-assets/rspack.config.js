const { ConcatSource, RawSource } = require("webpack-sources");

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	entry: {
		main: "./index.js"
	},
	devtool: "source-map",
	plugins: [
		{
			name: "test",
			apply(compiler) {
				compiler.hooks.compilation.tap("compilation", compilation => {
					compilation.hooks.processAssets.tapPromise(
						{
							name: "processAssets1",
							stage: compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_SUMMARIZE
						},
						async assets => {
							const dup = "dup.txt";
							assets[dup] = new RawSource(dup);
							const beforeDelete = new RawSource(Object.keys(assets).join(","));
							delete assets[dup];
							const afterDelete = new RawSource(Object.keys(assets).join(","));
							assets["assets-keys.txt"] = new ConcatSource(
								beforeDelete,
								"\n",
								afterDelete
							);
						}
					);
				});
			}
		}
	]
};
