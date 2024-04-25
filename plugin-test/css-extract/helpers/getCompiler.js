const path = require("path");
const { rspack: webpack } = require("@rspack/core");
const { createFsFromVolume, Volume } = require("memfs");
const { CssExtractRspackPlugin: MiniCssExtractPlugin } = require("@rspack/core");

module.exports = (fixture, loaderOptions = {}, config = {}) => {
	const { outputFileSystem, ...cnfg } = config;

	const fullConfig = {
		mode: "development",
		devtool: cnfg.devtool || false,
		context: path.resolve(__dirname, "../fixtures"),
		entry: path.resolve(__dirname, "../fixtures", fixture),
		output: {
			path: path.resolve(__dirname, "../outputs"),
			filename: "[name].bundle.js",
			chunkFilename: "[name].chunk.js"
		},
		module: {
			rules: [
				{
					test: /\.css$/i,
					rules: [
						{
							loader: MiniCssExtractPlugin.loader,
							options: loaderOptions || {}
						},
						{
							loader: "css-loader"
						}
					]
				}
			].concat({
				test: /\.svg$/,
				type: "asset/resource",
				generator: {
					filename: "[name][ext]"
				}
			})
		},
		plugins: [
			new MiniCssExtractPlugin({
				// Options similar to the same options in webpackOptions.output
				// both options are optional
				filename: "[name].css",
				chunkFilename: "[id].css"
			})
		],
		experiments: {
			css: false,
			rspackFuture: {
				newTreeshaking: true
			}
		},
		...cnfg
	};

	const compiler = webpack(fullConfig);

	if (!outputFileSystem) {
		compiler.outputFileSystem = createFsFromVolume(new Volume());
	} else {
		compiler.outputFileSystem = outputFileSystem;
	}

	return compiler;
};
