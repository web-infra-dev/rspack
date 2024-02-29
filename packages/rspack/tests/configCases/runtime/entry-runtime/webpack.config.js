module.exports = {
	entry: {
		main: {
			import: "./index.js",
			runtime: "runtime"
		}
	},
	output: {
		filename: '[name].js'
	}
};
