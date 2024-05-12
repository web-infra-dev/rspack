/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	entry: "./index.ts",
	module: {
		rules: [
			{
				test: /\.([cm]?ts|tsx)$/,
				use: [{ loader: "ts-loader" }],
			}
		]
	}
};
