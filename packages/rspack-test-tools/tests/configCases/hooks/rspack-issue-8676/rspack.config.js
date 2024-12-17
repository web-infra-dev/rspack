class Plugin {
	apply(compiler) {
		let testRuned = false;
		compiler.hooks.assetEmitted.tap("test", (filename, info) => {
			const { targetPath, content } = info;
			if (targetPath.endsWith(".css")) {
				testRuned = true;
				expect(content).toBeDefined();
				expect(filename.endsWith(".css?v=2")).toBeTruthy();
			}
		});
		compiler.hooks.done.tap("test", () => {
			expect(testRuned).toBeTruthy();
		});
	}
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	target: "web",
	mode: "development",
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "asset/resource"
			}
		]
	},
	plugins: [new Plugin()]
};
