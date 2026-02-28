it("should pass query to the loader", () => {
	let result = require("./a?resourcequery");
	expect(result).toEqual({
		resourceQuery: "?resourcequery",
		// Formatted by prettier
		prev: 'module.exports = "a";\n'
	});
});

it("should pass empty query to the loader", () => {
	let result = require("./b");
	expect(result).toEqual({
		resourceQuery: "",
		// Formatted by prettier
		prev: 'module.exports = "b";\n'
	});
});
