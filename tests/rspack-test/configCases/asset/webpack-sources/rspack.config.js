/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	devtool: "source-map",
	module: {
		rules: [
			{
				test: /\.png$/,
				type: "asset/resource",
				generator: {
					filename: "[name][ext]"
				}
			}
		]
	},
	plugins: [
		compiler => {
			compiler.hooks.thisCompilation.tap("PLUGIN", compilation => {
				compilation.hooks.processAssets.tap(
					{
						name: "PLUGIN",
						stage:
							compiler.webpack.Compilation
								.PROCESS_ASSETS_STAGE_OPTIMIZE_TRANSFER,
						additionalAssets: true
					},
					assets => {
						const { RawSource, SourceMapSource } = compiler.webpack.sources;

						expect(assets["img.png"]).toBeInstanceOf(RawSource);
						expect(assets["bundle0.js"]).toBeInstanceOf(SourceMapSource);
					}
				);
			});
		}
	]
};
