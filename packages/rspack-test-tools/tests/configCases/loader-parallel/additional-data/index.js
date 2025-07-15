it("should pass additional data between loaders", () => {
	const result = require("./a");
	expect(result).toEqual({
		a: "a",
		b: "b"
	});
});
