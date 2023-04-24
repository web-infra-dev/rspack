/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	target: "node",
	mode: "development",
	entry: {
		main: "./src/index.js"
	},
	builtins: {
		minify: false
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
				use: [{ loader: "less-loader" }],
				type: "css"
			},
			{
				test: /\.scss$/,
				use: [{ loader: "sass-loader" }],
				type: "css"
			},
			{
				test: /\.yaml$/,
				use: [{ loader: "yaml-loader" }]
			},
			{
				test: /\.styl$/,
				use: [{ loader: "stylus-loader" }],
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
				use: [
					{
						loader: "@svgr/webpack"
					},
					{
						loader: "file-loader"
					}
				],
				type: "javascript/auto"
			},
			{
				test: /\.txt/,
				use: [
					{
						loader: "raw-loader"
					}
				],
				type: "javascript/auto"
			},
			{
				test: /\.png$/,
				use: [
					{
						loader: "file-loader"
					}
				]
			}
		]
	}
};
