it("should pass null as additional data", () => {
	const result = require("./a");
	expect(result.value).toEqual("a");
});
