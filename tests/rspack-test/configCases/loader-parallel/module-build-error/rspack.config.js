/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /\.s[ac]ss$/i,
				use: [{ loader: "sass-loader", parallel: { maxWorkers: 4 }, options: {} }],
				type: "css"
			}
		]
	},
	experiments: {
		parallelLoader: true
	}
};
