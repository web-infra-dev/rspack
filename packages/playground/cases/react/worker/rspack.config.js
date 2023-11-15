/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
	context: __dirname,
	mode: "development",
	entry: "./src/index.jsx",
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
