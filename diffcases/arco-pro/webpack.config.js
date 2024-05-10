const path = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");

/** @type {import('webpack').Configuration} */
const config = {
	mode: "production",
	context: __dirname,
	entry: "./src/index.tsx",
	target: ["web", "es5"],
	module: {
		rules: [
			{
				test: /\.less$/,
				use: "less-loader",
				parser: {
					namedExports: false
				},
				generator: {
					exportsOnly: true
				},
				type: "css"
			},
			{
				test: /\.module\.less$/,
				use: "less-loader",
				parser: {
					namedExports: false
				},
				generator: {
					exportsOnly: true
				},
				type: "css/module"
			},
			{
				test: /\.svg$/,
				use: "@svgr/webpack"
			},
			{
				test: /\.(j|t)s$/,
				exclude: [/[\\/]node_modules[\\/]/],
				loader: "swc-loader",
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
				loader: "swc-loader",
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
								development: false,
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
				test: /\.png$/,
				type: "asset"
			}
		]
	},
	resolve: {
		alias: {
			"@": path.resolve(__dirname, "src"),
			// The default exported mock.js contains a minified [parser](https://github.com/nuysoft/Mock/blob/refactoring/src/mock/regexp/parser.js) with super deep binary
			// expression, which causes stack overflow for swc parser in debug mode.
			// Alias to the unminified version mitigates this problem.
			// See also <https://github.com/search?q=repo%3Aswc-project%2Fswc+parser+stack+overflow&type=issues>
			mockjs: require.resolve("./patches/mock.js")
		},
		extensions: [".js", ".jsx", ".ts", ".tsx", ".css", ".less"]
	},
	output: {
		publicPath: "/",
		filename: "[name].js",
		chunkFilename: "[name].js",
		cssChunkFilename: "[name].css",
		cssFilename: "[name].css"
	},
	optimization: {
		minimize: false, // Disabling minification because it takes too long on CI
		realContentHash: true,
		providedExports: true,
		usedExports: true,
		sideEffects: true,
		mangleExports: false,
		splitChunks: {
			cacheGroups: {
				someVendor: {
					chunks: "all",
					minChunks: 2,
					filename: "someVencor-[name].js",
					name: "vendor"
				}
			}
		}
	},
	experiments: {
		css: true
	},
	plugins: [
		new HtmlWebpackPlugin({
			title: "Arco Pro App",
			template: path.join(__dirname, "index.html"),
			favicon: path.join(__dirname, "public", "favicon.ico")
		})
	],
	infrastructureLogging: {
		debug: false
	}
};
module.exports = config;
