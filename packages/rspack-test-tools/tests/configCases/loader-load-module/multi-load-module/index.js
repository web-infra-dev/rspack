import value from "./loader.js!./value";

it("should multi load module works", () => {
	expect(value).toBe(1);
});
