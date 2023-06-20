/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	entry: {
		main: "./src/main.tsx"
	},
	module: {
		rules: [
			{
				test: /\.tsx$/,
				use: [
					{
						loader: "babel-loader",
						options: {
							"presets": ["@babel/preset-typescript"],
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
module.exports = config;
