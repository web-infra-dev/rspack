import value from "./loader.mjs!./value";

it("should multi import module works", () => {
	expect(value).toBe(2);
});
