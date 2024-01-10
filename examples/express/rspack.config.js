const { RunScriptWebpackPlugin } = require("run-script-webpack-plugin");

/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	target: "node",
	entry: {
		main: ["webpack/hot/poll?100", "./src/main.ts"]
	},
	resolve: {
		extensions: ["...", ".ts", ".tsx", ".jsx"]
	},
	module: {
		rules: [
			{
				test: /\.ts$/,
				use: {
					loader: "builtin:swc-loader",
					options: {
						jsc: {
							parser: {
								syntax: "typescript",
								decorators: true
							}
						}
					}
				}
			}
		]
	},
	optimization: {
		minimize: false
	},
	externalsType: "commonjs",
	plugins: [
		!process.env.BUILD &&
			new RunScriptWebpackPlugin({
				name: "main.js",
				autoRestart: false
			})
	].filter(Boolean),
	devServer: {
		devMiddleware: {
			writeToDisk: true
		}
	}
};
module.exports = config;
