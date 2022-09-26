/**
 * @type {import('webpack').Configuration}
 */
module.exports = {
	mode: "none",
	cache: {
		type: "filesystem"
	},
	context: __dirname,
	entry: {
		main: "./src/index.js"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "asset"
			},
			{
				test: /\.(svg|png|jpg)$/,
				type: "asset"
			}
		]
	}
};
