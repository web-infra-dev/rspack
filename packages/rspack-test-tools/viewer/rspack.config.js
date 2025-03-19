const { rspack, experiments } = require("../../rspack");
const path = require("node:path");
const MonacoWebpackPlugin = require("monaco-editor-webpack-plugin");

const prod = process.env.NODE_ENV === "production";

module.exports = {
	entry: {
		diff: "./entries/diff.tsx"
	},
	resolve: {
		extensions: [".js", ".jsx", ".tsx", ".ts", ".css", ".less"],
		tsConfig: path.resolve(__dirname, "./tsconfig.json")
	},
	devtool: false,
	output: {
		globalObject: "self",
		filename: "[name].bundle.js",
		path: path.resolve(__dirname, "../template")
	},
	module: {
		rules: [
			{
				test: /\.(j|t)s$/,
				exclude: [/[\\/]node_modules[\\/]/],
				loader: "builtin:swc-loader",
				options: {
					sourceMaps: false,
					jsc: {
						parser: {
							syntax: "typescript"
						},
						externalHelpers: true
					},
					env: {
						targets: "Chrome >= 48"
					}
				}
			},
			{
				test: /\.(j|t)sx$/,
				loader: "builtin:swc-loader",
				exclude: [/[\\/]node_modules[\\/]/],
				options: {
					sourceMaps: false,
					jsc: {
						parser: {
							syntax: "typescript",
							tsx: true
						},
						transform: {
							react: {
								runtime: "automatic",
								development: !prod,
								refresh: false
							}
						},
						externalHelpers: true
					},
					env: {
						targets: "Chrome >= 48"
					}
				}
			},
			{
				test: /\.ttf$/,
				type: "asset/resource"
			}
		]
	},
	experiments: {
		css: true
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./templates/diff.html",
			filename: "[name].html"
		}),
		new MonacoWebpackPlugin({
			languages: ["javascript"]
		})
	]
};
