const { Compilation, CssExtractRspackPlugin } = require("@rspack/core");

const compression = exts => compiler => {
	compiler.hooks.thisCompilation.tap("Test", compilation => {
		compilation.hooks.processAssets.tap(
			{
				name: "Test",
				stage: Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_TRANSFER
			},
			() => {
				for (const asset of compilation.getAssets()) {
					for (const ext of exts) {
						const newFile = `${asset.name}${ext}`;
						compilation.emitAsset(newFile, asset.source);
						compilation.updateAsset(asset.name, asset.source, {
							related: {
								compressed: ["...", newFile]
							}
						});
					}
				}
			}
		);
	});
};

const base = name => ({
	name,
	mode: "development",
	devtool: "source-map",
	entry: "./index",
	output: {
		filename: `${name}-[name].js`
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					CssExtractRspackPlugin.loader,
					{
						loader: "css-loader",
						options: {
							sourceMap: true
						}
					}
				]
			}
		]
	},
	plugins: [
		new CssExtractRspackPlugin({
			filename: `${name}-[name].css`
		}),
		compression([".br", ".gz"])
	]
});

const baseStats = {
	entrypoints: false,
	modules: false,
	timings: false,
	version: false,
	hash: false,
	builtAt: false,
	errorsCount: false,
	warningsCount: false
};

/** @type {import("@rspack/core").Configuration} */
module.exports = [
	{
		...base("default"),
		stats: {
			...baseStats
		}
	},
	{
		...base("relatedAssets"),
		stats: {
			...baseStats,
			relatedAssets: true
		}
	},
	{
		...base("exclude1"),
		stats: {
			...baseStats,
			relatedAssets: true,
			excludeAssets: /\.(gz|br)$/
		}
	},
	{
		...base("exclude2"),
		stats: {
			...baseStats,
			relatedAssets: true,
			excludeAssets: /\.map$/
		}
	},
	{
		...base("exclude3"),
		stats: {
			...baseStats,
			relatedAssets: true,
			excludeAssets: /chunk/
		}
	}
];
