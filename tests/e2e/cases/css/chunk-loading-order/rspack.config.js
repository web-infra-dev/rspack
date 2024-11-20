const rspack = require("@rspack/core");

module.exports = {
	context: __dirname,
	mode: "development",
	entry: "./src/index.js",
	devServer: {
		hot: true
	},
	stats: "none",
	plugins: [new rspack.HtmlRspackPlugin()],
	watchOptions: {
		poll: 1000
	},
	optimization: {
		splitChunks: {
			minSize: 0,
			cacheGroups: {
				a: {
					test: /a.css/,
					name: "a"
				},
				b: {
					test: /b.css/,
					name: "b"
				}
			}
		}
	},
	experiments: {
		css: true
	}
};
