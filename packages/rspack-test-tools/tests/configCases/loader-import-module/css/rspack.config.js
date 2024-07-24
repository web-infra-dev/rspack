/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		publicPath: "/public/"
	},
	entry: "./index.js",
	module: {
		parser: {
			javascript: {
				// url: "relative"
			}
		},
		rules: [
			{
				dependency: "url",
				issuer: /stylesheet\.js$/,
				type: "asset/resource",
				generator: {
					filename: "assets/[name][ext][query]"
				}
			},
			{
				oneOf: [
					{
						test: /other-stylesheet\.js$/,
						loader: "./loader",
						options: {
							publicPath: "/other/",
							baseUri: "my-schema://base"
						},
						type: "asset/source"
					},
					{
						test: /stylesheet\.js$/,
						loader: "./loader",
						options: {
							baseUri: "my-schema://base"
						},
						type: "asset/source"
					}
				]
			},
			{
				test: /\.jpg$/,
				type: "asset/resource",
				generator: {
					filename: "assets/[name][ext]"
				}
			}
		]
	},
	plugins: [
		compiler =>
			compiler.hooks.done.tap("test case", stats => {
				try {
					expect(stats.compilation.getAsset("assets/file.png")).toHaveProperty(
						"info",
						expect.objectContaining({ sourceFilename: "file.png" })
					);
					expect(stats.compilation.getAsset("assets/file.jpg")).toHaveProperty(
						"info",
						expect.objectContaining({ sourceFilename: "file.jpg" })
					);
					const { auxiliaryFiles } = stats.compilation.namedChunks.get("main");
					expect(auxiliaryFiles).toContain("assets/file.png");
				} catch (e) {
					console.log(stats.toString({ colors: true, orphanModules: true }));
					throw e;
				}
			})
	],
};
