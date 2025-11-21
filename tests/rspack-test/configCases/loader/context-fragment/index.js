it("should pass fragment to the loader", () => {
	let result = require("./a#resourcefragment");
	expect(result).toEqual({
		resourceFragment: "#resourcefragment",
		// Formatted by prettier
		prev: 'module.exports = "a";\n'
	});
});

it("should pass empty fragment to the loader", () => {
	let result = require("./b");
	expect(result).toEqual({
		resourceFragment: "",
		// Formatted by prettier
		prev: 'module.exports = "b";\n'
	});
});
