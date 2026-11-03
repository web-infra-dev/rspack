it("should preserve JSON property names when a prototype method is used", () => {
	expect(require("./bridge.cjs").data.longProperty).toBe(42);
	expect(require("./bridge.cjs").data.nested.longProperty).toBe(43);
	expect(require("./bridge.cjs").data.hasOwnProperty("longProperty")).toBe(true);
});

it("should preserve nested JSON property names when a prototype method is used", () => {
	expect(require("./bridge.cjs").nested.object.longProperty).toBe(43);
	expect(require("./bridge.cjs").nested.object.hasOwnProperty("longProperty")).toBe(true);
	expect(require("./bridge.cjs").nested.array[0].longProperty).toBe(44);
	expect(require("./bridge.cjs").nested.array[0].hasOwnProperty("longProperty")).toBe(true);
});
