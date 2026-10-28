it("should not deadlock when using loadModule", () => {
	const result = require("./loader.mjs!");
	expect(result).toMatch(/console.log\(42\)/);
});
