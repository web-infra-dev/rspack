const path = require("path");
const rspack = require("@rspack/core");

// Import React Refresh plugin - similar to Next.js
const ReactRefreshPlugin = require("@rspack/plugin-react-refresh");

module.exports = {
	mode: "development",
	entry: "./index.js",
	output: {
		path: path.resolve(__dirname, "dist"),
		filename: "bundle.js"
	},
	externalsPresets: {},
	module: {
		rules: [
			{
				test: /\.(js|jsx|mjs)$/,
				exclude: /node_modules/,
				use: [
					{
						loader: "builtin:swc-loader",
						options: {
							jsc: {
								transform: {
									react: {
										runtime: "automatic",
										development: true,
										refresh: true
									}
								},
								parser: {
									syntax: "ecmascript",
									jsx: true
								}
							}
						}
					}
				]
			}
		]
	},
	experiments: {
		buildHttp: {
			allowedUris: ["https://esm.sh"],
			cacheLocation: path.resolve(__dirname, "lock-files")
		}
	},
	plugins: [
		// Add React Refresh Plugin
		new ReactRefreshPlugin(),
		// Required for HMR
		new rspack.HotModuleReplacementPlugin()
	],
	resolve: {
		extensions: [".js", ".jsx", ".mjs"]
	},
	devServer: {
		hot: true
	}
};
