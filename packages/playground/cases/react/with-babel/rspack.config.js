const rspack = require("@rspack/core");
const ReactRefreshPlugin = require("@rspack/plugin-react-refresh");

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
	experiments: {
		rspackFuture: {
			disableTransformByDefault: true
		}
	},
	context: __dirname,
	mode: "development",
	module: {
		rules: [
			{
				test: /\.jsx$/,
				use: {
					loader: "babel-loader",
					options: {
						presets: [["@babel/preset-react", { runtime: "automatic" }]],
						plugins: [require.resolve("react-refresh/babel")]
					}
				}
			}
		]
	},
	plugins: [
		new rspack.HtmlRspackPlugin({ template: "./src/index.html" }),
		new ReactRefreshPlugin()
	],
	entry: "./src/index.jsx",
	devServer: {
		hot: true
	},
	cache: false,
	stats: "none",
	infrastructureLogging: {
		debug: false
	},
	watchOptions: {
		poll: 1000
	}
};
