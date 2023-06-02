/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	entry: {
		main: "./src/index.js"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					{
						loader: "postcss-loader",
						options: {
							postcssOptions: {
								plugins: {
									tailwindcss: {},
									autoprefixer: {}
								}
							}
						}
					}
				],
				type: "css"
			}
		]
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		]
	}
};
module.exports = config;
