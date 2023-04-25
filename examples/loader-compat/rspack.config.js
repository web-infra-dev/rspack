/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	entry: {
		main: "./src/index.js"
	},
	builtins: {
		html: [
			{
				template: "./index.html"
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
								plugins: ["autoprefixer"]
							}
						}
					}
				],
				type: "css"
			},
			{
				test: /\.js$/,
				use: [
					{
						loader: "babel-loader",
						options: {
							presets: [["@babel/preset-env", { targets: "defaults" }]]
						}
					},
					{
						loader: "source-map-loader"
					}
				]
			},
			{
				test: /\.less$/,
				use: [
					{
						loader: "style-loader",
						options: {
							esModule: false
						}
					},
					"css-loader",
					"less-loader"
				]
			},
			{
				test: /\.scss$/,
				use: [
					{ loader: "style-loader", options: { esModule: false } },
					"css-loader",
					"sass-loader"
				]
			},
			{
				test: /\.yaml$/,
				use: ["yaml-loader"]
			},
			{
				test: /\.styl$/,
				use: ["stylus-loader"],
				type: "css"
			},
			{
				test: /\.mdx?$/,
				use: [
					{
						loader: "@mdx-js/loader",
						options: {}
					}
				]
			},
			{
				test: /\.svg$/,
				use: ["@svgr/webpack", "file-loader"],
				type: "javascript/auto"
			},
			{
				test: /\.txt/,
				use: ["raw-loader"],
				type: "javascript/auto"
			},
			{
				test: /\.png$/,
				use: ["file-loader"]
			}
		]
	}
};
