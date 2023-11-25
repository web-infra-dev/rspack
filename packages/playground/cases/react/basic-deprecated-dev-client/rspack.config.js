/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
	context: __dirname,
	mode: "development",
	module: {
		// add this to test react refresh runtime shouldn't inject runtime,see #3984
		rules: [
			{
				test: /\.js$/,
				type: "jsx"
			}
		]
	},
	entry: ["@rspack/dev-client/react-refresh-entry", "./src/index.jsx"],
	devServer: {
		hot: true
	},
	cache: false,
	stats: "none",
	infrastructureLogging: {
		debug: false
	},
	builtins: {
		provide: {
			$ReactRefreshRuntime$: [require.resolve("./react-refresh.js")]
		},
		html: [
			{
				template: "./src/index.html"
			}
		]
	},
	experiments: {
		rspackFuture: {
			disableTransformByDefault: false
		}
	},
	watchOptions: {
		poll: 1000
	}
};
