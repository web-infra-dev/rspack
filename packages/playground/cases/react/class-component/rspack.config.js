const rspack = require("@rspack/core");

module.exports = {
	context: __dirname,
	mode: "development",
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
