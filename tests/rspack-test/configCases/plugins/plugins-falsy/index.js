const test = require("./a");

it("should work with falsy plugins", function () {
	expect(test.default).toBe("test");
});
