class Plugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("Test", compilation => {
			const chunks = compilation.chunks;

			compilation.hooks.processAssets.tap("Test", () => {
				expect(chunks).toBe(compilation.chunks);
				expect(chunks.size).toBe(1);
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
