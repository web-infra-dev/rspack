it("should pass query to the loader", () => {
	let result = require("./a?resourcequery");
	expect(result).toEqual({
		resourceQuery: "?resourcequery",
		prev: 'module.exports = "a";'
	});
});

it("should pass empty query to the loader", () => {
	let result = require("./b");
	expect(result).toEqual({
		resourceQuery: "",
		prev: 'module.exports = "b";'
	});
});
