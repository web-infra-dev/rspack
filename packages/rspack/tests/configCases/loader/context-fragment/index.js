it("should pass fragment to the loader", () => {
	let result = require("./a#resourcefragment");
	expect(result).toEqual({
		resourceFragment: "#resourcefragment",
		prev: 'module.exports = "a";'
	});
});

it("should pass empty fragment to the loader", () => {
	let result = require("./b");
	expect(result).toEqual({
		resourceFragment: "",
		prev: 'module.exports = "b";'
	});
});
