/** @type { import('@rspack/core').RspackOptions } */

module.exports = {
	context: __dirname,
	mode: "development",
	entry: "./src/index.js",
	stats: "none",
	builtins: {
		html: [
			{
				template: "./src/index.html"
			}
		],
		define: {
			"process.env.NODE_ENV": JSON.stringify("development")
		}
	},
	output: {
		crossOriginLoading: "anonymous"
	},
	devServer: {
		port: 3000
	}
};
