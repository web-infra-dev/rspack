const path = require("path");

module.exports = {
	context: __dirname,
	entry: {
		main: "./src/main.jsx"
	},
	builtins: {
		html: [
			{
				template: "./src/index.html"
			}
		]
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
	}
};
