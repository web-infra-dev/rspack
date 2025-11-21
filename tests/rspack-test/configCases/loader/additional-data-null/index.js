it("should pass null as additional data", () => {
	let result = require("./a");
	expect(result.value).toEqual("a");
});
