/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	incremental: false,
	module: {
		rules: [
			{
				test: /file\.js$/,
				loader: "./loader.js",
				options: {
					files: []
				}
			}
		]
	},
	cache: {
		type: "persistent"
	},
	plugins: [
		{
			updateIndex: 0,
			apply(compiler) {
				compiler.hooks.done.tap("Test", () => {
					const options = compiler.options.module.rules[0].options;
					expect(options.files.length).toBe(1);
					options.files = [];
					this.updateIndex++;
				});
			}
		}
	]
};
