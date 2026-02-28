import value from "./loader.js!./value";

it("should multi import module works", () => {
	expect(value).toBe(2);
});
