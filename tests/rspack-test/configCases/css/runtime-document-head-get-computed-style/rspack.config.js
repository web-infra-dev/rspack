/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	mode: "development",
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/auto"
			}
		]
	},
	output: {
		uniqueName: "test"
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.compilation.tap("Test", compilation => {
					compilation.hooks.processAssets.tap(
						{
							name: "Test",
							stage:
								compiler.rspack.Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE
						},
						assets => {
							const name = "bundle0.css";
							const code = assets[name].source();

							compilation.updateAsset(
								name,
								new compiler.rspack.sources.RawSource(
									`${code}\n\n.after-head { color: red; }`
								)
							);
						}
					);
				});
			}
		}
	],

};
