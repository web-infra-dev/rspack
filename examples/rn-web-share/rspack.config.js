/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	mode: "development",
	entry: {
		main: "./src/index.ts"
	},
	target: "node",
	module: {
		rules: [
			{
				test: /\.native\.ts$/,
				use: [
					{
						loader: "null-loader"
					}
				]
			}
		]
	}
};
module.exports = config;
