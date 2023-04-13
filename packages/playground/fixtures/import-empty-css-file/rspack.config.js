/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
	context: __dirname,
	mode: "development",
	entry: "./src/index.js",
	devServer: {
		hot: true
	},
	cache: false,
	stats: "none",
	infrastructureLogging: {
		debug: false
	},
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
	watchOptions: {
		poll: 1000
	}
};
