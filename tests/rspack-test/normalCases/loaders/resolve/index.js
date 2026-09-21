it("should be possible to create resolver with different options", () => {
	const result = require("./loader.mjs!");
	expect(result).toEqual({
		one: "index.js",
		two: "index.xyz",
		three: "index.js",
		four: "index.xyz",
		five: "index.js"
	});
});
