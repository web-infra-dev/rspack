const { HotModuleReplacementPlugin } = require("@rspack/core");

module.exports = {
	mode: "development",
	entry: {
		main: "./example.js"
	},
	cache: true,
	lazyCompilation: true,
	devServer: {
		hot: true,
		devMiddleware: {
			publicPath: "/dist/"
		}
	},
	module: {
		parser: {
			javascript: {
				exportsPresence: "auto",
			}
		}
	},
	plugins: [new HotModuleReplacementPlugin()]
};
