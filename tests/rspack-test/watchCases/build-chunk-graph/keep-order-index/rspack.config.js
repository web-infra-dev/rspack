const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [new rspack.CssExtractRspackPlugin({ ignoreOrder: true })],
	module: {
		rules: [
			{
				test: /\.css$/,
        type: 'javascript/auto',
				use: [rspack.CssExtractRspackPlugin.loader, "css-loader"]
			}
		]
	},
	experiments: {
		incremental: {
			buildChunkGraph: true
		}
	},
	optimization: {
		splitChunks: {
			cacheGroups: {
				styles: {
					name: "styles",
					chunks: "all",
					test: /\.css$/,
					enforce: true
				}
			}
		}
	},
	experiments: {
		incremental: true
	}
};
