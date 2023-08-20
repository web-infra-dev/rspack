/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	mode: "development",
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
								plugins: [
									["autoprefixer"],
									[
										"postcss-plugin-px2rem",
										{
											rootValue: 100,
											unitPrecision: 5,
											propWhiteList: [],
											propBlackList: [],
											exclude: false,
											selectorBlackList: [],
											ignoreIdentifier: false,
											replace: true,
											mediaQuery: false,
											minPixelValue: 0
										}
									]
								]
							}
						}
					}
				]
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
