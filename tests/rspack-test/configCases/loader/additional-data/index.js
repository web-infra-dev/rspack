it("should pass additional data between loaders", () => {
	let result = require("./a");
	expect(result).toEqual({
		a: "a",
		b: "b"
	});
});
