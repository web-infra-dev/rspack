module.exports = {
	context: __dirname,
	mode: "development",
	entry: {
		main: "./src/index.js"
	},
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
	watchOptions: {
		poll: 1000
	}
};
