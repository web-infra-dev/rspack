/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		function (compiler) {
			compiler.hooks.make.tap("TestPlugin", () => {
				throw new Error("Test error in make hook");
			});
			compiler.hooks.done.tap("TestPlugin", () => {
				// Should not continue run the compilation after throw error in make hook
				expect(true).toBe(false);
			});
		}
	]
};
