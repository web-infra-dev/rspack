/** @type { import('@rspack/core').RspackOptions } */
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
	watchOptions: {
		poll: 1000
	}
};
