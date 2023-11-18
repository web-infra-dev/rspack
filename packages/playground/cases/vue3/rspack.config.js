const { DefinePlugin, HtmlRspackPlugin } = require("@rspack/core");
const { VueLoaderPlugin } = require("vue-loader");

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
	context: __dirname,
	mode: "development",
	entry: "./src/main.js",
	devServer: {
		hot: true
	},
	plugins: [
		new VueLoaderPlugin(),
		new HtmlRspackPlugin({
			template: "./src/index.html"
		}),
		new DefinePlugin({
			__VUE_OPTIONS_API__: JSON.stringify(true),
			__VUE_PROD_DEVTOOLS__: JSON.stringify(false)
		})
	],
	module: {
		rules: [
			{
				test: /\.vue$/,
				loader: "vue-loader",
				options: {
					experimentalInlineMatchResource: true
				}
			},
			{
				test: /\.ts$/,
				loader: "builtin:swc-loader",
				options: {
					jsc: {
						parser: {
							syntax: "typescript"
						}
					}
				}
			}
		]
	},
	cache: false,
	stats: "error",
	infrastructureLogging: {
		debug: false
	},
	watchOptions: {
		poll: 1000
	},
	experiments: {
		rspackFuture: {
			disableTransformByDefault: true
		}
	}
};
