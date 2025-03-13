module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /lib\.js/,
				loader: "./loader-in-worker.js",
				parallel: true,
				options: {}
			}
		]
	},
	experiments: {
		parallelLoader: false
	}
};
