it("should pass additional data between loaders if builtin loader passes through its additional data", () => {
	let result = require("./a");
	expect(result).toEqual({
		a: "a",
		b: "b"
	});
});
