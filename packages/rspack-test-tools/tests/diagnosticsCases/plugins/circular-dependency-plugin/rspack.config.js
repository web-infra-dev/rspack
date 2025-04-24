const { CircularDependencyRspackPlugin } = require("@rspack/core");
const startFn = jest.fn();
const endFn = jest.fn();

module.exports = {
	entry: {
		aa: "./require-circular/d.js",
		bb: "./import-circular/index.js",
		cc: "./no-cycle/index.js",
		dd: "./ignore-circular/a.js"
	},
	plugins: [
		new CircularDependencyRspackPlugin({
			failOnError: false,
			exclude: /ignore-circular/,
			onStart(_compilation) {
				expect(typeof _compilation.errors === "object").toBeTruthy();
				expect(typeof _compilation.errors.push === "function").toBeTruthy();
				startFn();
			},
			onEnd(_compilation) {
				endFn();
			}
		}),
		{
			apply(compiler) {
				compiler.hooks.done.tap("done", () => {
					expect(startFn).toHaveBeenCalled();
					expect(endFn).toHaveBeenCalled();
				});
			}
		}
	]
};
