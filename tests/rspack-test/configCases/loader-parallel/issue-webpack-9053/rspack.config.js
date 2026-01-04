/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /c\.js$/,
				use: [{ loader: "loader2", parallel: { maxWorkers: 4 }, options: {} }]
			},
			{
				test: /d\.js$/,
				use: [{ loader: "loader3", parallel: { maxWorkers: 4 }, options: {} }]
			}
		]
	},
	experiments: {
		parallelLoader: true
	}
};
