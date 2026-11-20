it("should keep an unused method that uses super", () => {
	const m = require("./super");
	expect(m.used).toBe("used");
	expect(m.usedExports).toEqual(["used", "usedExports"]);
});

it("should not treat a compound assignment as an object literal export", () => {
	const m = require("./compound");
	expect(typeof m).toBe("string");
});
