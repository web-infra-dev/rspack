/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	context: __dirname,
	entry: {
		main: "./src/main.jsx"
	},
	module: {
		rules: [
			{
				test: /\.jsx$/,
				use: [
					{
						loader: "babel-loader",
						options: {
							plugins: ["@vue/babel-plugin-jsx"]
						}
					}
				]
			},
			{
				test: /\.svg$/,
				type: "asset"
			}
		]
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		],
		define: {
			__VUE_OPTIONS_API__: JSON.stringify(true),
			__VUE_PROD_DEVTOOLS__: JSON.stringify(false)
		}
	}
};
