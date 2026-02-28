class Plugin {
	/**
	* @param {import("@rspack/core").Compiler} compiler
	*/
	apply(compiler) {
		compiler.hooks.compilation.tap("Test", compilation => {
			const chunks = compilation.chunks;

			compilation.hooks.processAssets.tap("Test", () => {
				expect(chunks).toBe(compilation.chunks);
				expect(chunks.size).toBe(1);

				const chunk = Array.from(chunks)[0];
				const mockFn = rstest.fn((value, value2, set) => {
					expect(value).toBe(chunk);
					expect(value2).toBe(chunk);
					expect(set).toBe(chunks);
				});
				chunks.forEach(mockFn);
				expect(mockFn).toHaveBeenCalledTimes(1);

				const entries = chunks.entries();
				expect(entries.next()).toStrictEqual({ value: [chunk, chunk], done: false });
				expect(entries.next()).toStrictEqual({ value: undefined, done: true });
			});
		});
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./index.js"
	},
	plugins: [new Plugin()]
};
