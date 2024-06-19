/** @type {import("@rspack/core").Configuration} */
module.exports = {
	resolve: {
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
										rootDir: __dirname
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
