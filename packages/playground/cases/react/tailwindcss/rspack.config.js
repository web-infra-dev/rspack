const path = require("path");
const rspack = require("@rspack/core");

module.exports = {
	context: __dirname,
	entry: {
		main: "./src/main.jsx"
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
									tailwindcss: {
										config: path.join(__dirname, "./tailwind.config.js")
									}
								}
							}
						}
					}
				],
				type: "css"
			}
		]
	},
	experiments: {
		rspackFuture: {
			disableTransformByDefault: false
		}
	},
	plugins: [new rspack.HtmlRspackPlugin({ template: "./src/index.html" })]
};
