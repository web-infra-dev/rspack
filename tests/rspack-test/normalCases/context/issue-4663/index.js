it("should evaluate `require.context` with template literals", () => {
	const requireAssets = require.context(`./assets`, true, /\.js$/iu);
	expect(requireAssets("./sample.js").hello).toBe("Hello, world!");
});

it("should evaluate `import.meta.webpackContext` with template literals", () => {
	const requireAssets = import.meta.webpackContext(`./assets`, {
		recursive: true,
		regExp: /\.js$/iu
	});
	expect(requireAssets("./sample.js").hello).toBe("Hello, world!");
});
