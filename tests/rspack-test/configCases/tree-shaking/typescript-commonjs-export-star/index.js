const { local, shadowed, used } = require("./barrel");

it("preserves explicit exports around a TypeScript CommonJS star reexport", () => {
	expect(local).toBe("local");
	expect(shadowed).toBe("barrel");
	expect(used).toBe("used");
});
