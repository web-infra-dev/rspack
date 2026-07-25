const first = require("./module-a");
const second = require("./module-b");

it("should share TypeScript default import fallbacks", () => {
	expect(first.default).toBe("a");
	expect(second.default).toBe("b");
	expect(second.marker).toBe(true);
});
