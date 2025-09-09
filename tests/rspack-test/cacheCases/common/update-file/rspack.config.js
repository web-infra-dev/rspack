/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
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
	experiments: {
		cache: {
			type: "persistent"
		}
	},
	plugins: [
		{
			updateIndex: 0,
			apply(compiler) {
				compiler.hooks.done.tap("Test", () => {
					const options = compiler.options.module.rules[0].options;
					if (this.updateIndex == 0) {
						expect(options.files.length).toBe(1);
					}
					if (this.updateIndex == 1) {
						expect(options.files.length).toBe(1);
					}
					if (this.updateIndex == 2) {
						expect(options.files.length).toBe(0);
					}
					if (this.updateIndex == 3) {
						expect(options.files.length).toBe(0);
					}
					if (this.updateIndex == 4) {
						expect(options.files.length).toBe(1);
					}
					options.files = [];
					this.updateIndex++;
				});
			}
		}
	]
};
