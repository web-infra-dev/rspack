let step = 0;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: () => {
		if (step === 0) {
			return {
				bundle0: "./index.js"
			};
		} else if (step === 1) {
			return {
				bundle0: "./index.js",
				bundle1: "./index.js"
			};
		} else {
			throw new Error("no more steps");
		}
	},
	output: {
		filename: "[name].js"
	},
	plugins: [
		function (compiler) {
			compiler.hooks.done.tap("test", () => {
				if (step === 0) {
					process.nextTick(() => {
						step += 1;
						compiler.watching.invalidate();
					});
				}
			});
			compiler.hooks.thisCompilation.tap("MyPlugin", compilation => {
				compilation.hooks.processAssets.tap("MyPlugin", () => {
					if (step === 0) {
						expect([...compilation.entrypoints.keys()]).toEqual(["bundle0"]);
					} else if (step === 1) {
						expect([...compilation.entrypoints.keys()]).toEqual([
							"bundle0",
							"bundle1"
						]);
					} else {
						throw new Error("no more steps");
					}
				});
			});
		}
	]
};
