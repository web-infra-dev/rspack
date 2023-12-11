const path = require("path");
const rspack = require("@rspack/core");
const { default: HtmlPlugin } = require("@rspack/plugin-html");

/** @type {import('@rspack/cli').Configuration} */
const config = {
	mode: 'production',
	context: __dirname,
	entry: "./src/index.tsx",
	target: ["web", "es5"],
	module: {
		rules: [
			{
				test: /\.less$/,
				use: "less-loader",
				type: "css"
			},
			{
				test: /\.module\.less$/,
				use: "less-loader",
				type: "css/module"
			},
			{
				test: /\.svg$/,
				use: "@svgr/webpack"
			},
			{
				test: /\.(j|t)s$/,
				exclude: [/[\\/]node_modules[\\/]/],
				loader: "builtin:swc-loader",
				options: {
					sourceMap: false,
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
					sourceMap: false,
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
			mockjs: require.resolve("./patches/mock.js"),
			"@swc/helpers": require.resolve('@swc/helpers'),
		}
	},
	output: {
		publicPath: "/",
		filename: "[name].js",
		chunkFilename: "[name].js"
	},
	optimization: {
		minimize: false, // Disabling minification because it takes too long on CI
		realContentHash: true,
		usedExports: false,
		splitChunks: {
			cacheGroups: {
				someVendor: {
					chunks: "all",
					minChunks: 2
				}
			}
		}
	},
	plugins: [
		new HtmlPlugin({
			title: "Arco Pro App",
			template: path.join(__dirname, "index.html"),
			favicon: path.join(__dirname, "public", "favicon.ico")
		}),
	],
	infrastructureLogging: {
		debug: false
	},
	experiments: {
		css: true,
		rspackFuture: {
			disableTransformByDefault: true
		}
	},
	builtins: {
		css: {
			modules: {
				exportsOnly: true,
				localIdentName: "example-arco-design-pro---[path][name][ext]-[local]",
			},
		},
	},
};
module.exports = config;
