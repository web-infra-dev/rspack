const rspack = require("@rspack/core");

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
	entry: [
		"@rspack/plugin-react-refresh/react-refresh-entry",
		"./src/index.jsx"
	],
	devServer: {
		hot: true
	},
	cache: false,
	stats: "none",
	infrastructureLogging: {
		debug: false
	},
	experiments: {
		rspackFuture: {
			disableTransformByDefault: false
		}
	},
	watchOptions: {
		poll: 1000
	},
	plugins: [
		new rspack.HtmlRspackPlugin({ template: "./src/index.html" }),
		new rspack.ProvidePlugin({
			$ReactRefreshRuntime$: [require.resolve("./react-refresh.js")]
		})
	]
};
