it("should pass source map between loaders", () => {
	let result = require("./a");
	expect(result).toEqual({
		version: 3,
		sources: ["index.js"],
		mappings: "AAAA"
	});
});
