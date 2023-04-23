module.exports = {
	context: __dirname,
	entry: {
		main: "./src/index.jsx"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css"
			},
			{
				test: /\.module.css$/,
				type: "css/module"
			},
			{
				test: /\.svg$/,
				type: "asset/resource"
			},
			{
				test: /\.jsx$/,
				use: [
					{
						loader: "babel-loader",
						options: {
							presets: [["solid"]],
							plugins: ["solid-refresh/babel"]
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
		],
		react: {
			refresh: false
		}
	}
};
