const { VueLoaderPlugin } = require("vue-loader");

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
	context: __dirname,
	mode: "development",
	entry: "./src/main.js",
	builtins: {
		html: [
			{
				template: "./src/index.html"
			}
		],
		define: {
			__VUE_OPTIONS_API__: JSON.stringify(true),
			__VUE_PROD_DEVTOOLS__: JSON.stringify(false)
		}
	},
	devServer: {
		hot: true
	},
	plugins: [new VueLoaderPlugin()],
	module: {
		rules: [
			{
				test: /\.vue$/,
				loader: "vue-loader",
				options: {
					experimentalInlineMatchResource: true
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
	}
};
