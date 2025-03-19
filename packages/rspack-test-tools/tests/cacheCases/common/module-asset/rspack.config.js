/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /index\.js$/,
				loader: "./loader.js"
			}
		]
	},
	experiments: {
		cache: {
			type: "persistent"
		}
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.done.tap("Test", function (stats) {
					let s = stats.toJson({
						assets: true
					});
					expect(s.assets.some(item => item.name === "a.txt")).toBeTruthy();
				});
			}
		}
	]
};
