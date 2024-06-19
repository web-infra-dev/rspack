const { resolve } = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	resolve: {
		alias: {
			[resolve(__dirname, "./custom/MyComponent.graphql.ts")]: resolve(
				__dirname,
				"./mock.js"
			)
		},
		extensions: ["...", ".ts", ".tsx", ".jsx"]
	},
	module: {
		rules: [
			{
				test: /\.js$/,
				loader: "builtin:swc-loader",
				options: {
					jsc: {
						experimental: {
							plugins: [
								[
									"@swc/plugin-relay",
									{
										rootDir: __dirname,
										language: "typescript",
										artifactDirectory: "custom"
									}
								]
							]
						}
					},
				}
			}
		]
	}
};
