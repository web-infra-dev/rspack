const { omit: omitA } = require("./module-a");
const { omit: omitB } = require("./module-b");

it("should share TypeScript object rest fallbacks", () => {
	const kept = Symbol("kept");
	const excluded = Symbol("excluded");
	const source = { a: 1, b: 2, [kept]: 3, [excluded]: 4 };
	Object.defineProperty(source, "hidden", {
		enumerable: false,
		value: 5
	});

	const resultA = omitA(source, ["a", excluded]);
	expect(resultA.b).toBe(2);
	expect(resultA[kept]).toBe(3);
	expect(resultA).not.toHaveProperty("a");
	expect(resultA[excluded]).toBeUndefined();
	expect(resultA).not.toHaveProperty("hidden");

	expect(omitB({ c: 6, d: 7 }, ["d"])).toEqual({ c: 6 });
});
