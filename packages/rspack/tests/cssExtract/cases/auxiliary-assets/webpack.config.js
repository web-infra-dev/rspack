import { RspackCssExtractPlugin } from "../../../../src";

class AssetsPlugin {
	// eslint-disable-next-line class-methods-use-this
	apply(compiler) {
		compiler.hooks.emit.tapAsync("AssetsPlugin", (compilation, cb) => {
			const stats = compilation.getStats().toJson({
				all: true
			});

			const { RawSource } = compiler.webpack.sources;

			for (const file of stats.entrypoints.main.auxiliaryAssets) {
				const newFile = `auxiliaryAssets-${file.name}`;
				compilation.emitAsset(newFile, new RawSource(newFile), {});
			}

			cb();
		});
	}
}

module.exports = {
	entry: "./index.js",
	mode: "development",
	output: {
		publicPath: "/"
	},
	optimization: {
		sideEffects: true
	},
	module: {
		rules: [
			{
				test: /\.(woff2?|ttf|eot|otf|png|jpe?g|gif|ico|svg|webp)$/,
				type: "asset"
			},
			{
				test: /\.css$/,
				use: [
					{
						loader: RspackCssExtractPlugin.loader,
						options: {
							esModule: true
						}
					},
					{
						loader: "css-loader",
						options: {
							esModule: true
						}
					}
				]
			}
		]
	},
	plugins: [
		new RspackCssExtractPlugin({
			filename: "[name].css",
			chunkFilename: "[id].[name].css"
		}),
		new AssetsPlugin()
	]
};
