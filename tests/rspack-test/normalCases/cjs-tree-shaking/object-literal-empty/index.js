it("should keep an empty module.exports object literal", () => {
	const m = require("./empty");
	expect(m).toEqual({});
	expect(m.missing).toBe(undefined);
});

it("should track exports assigned after an empty module.exports object literal", () => {
	const m = require("./assigned");
	expect(m.used).toBe("used");
	expect(m.usedExports).toEqual(["used", "usedExports"]);
});

it("should bailout on an empty module.exports object literal that is not a statement", () => {
	const m = require("./nested");
	expect(m.used).toBe("used");
	expect(m.usedExports).toBe(null);
});
