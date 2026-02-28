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
	cache: {
		type: "persistent"
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.done.tap("Test", function (stats) {
					let s = stats.toJson({
						all: true
					});
					expect(s.assets.some(item => item.name === "a.txt")).toBeTruthy();
				});
			}
		}
	]
};
