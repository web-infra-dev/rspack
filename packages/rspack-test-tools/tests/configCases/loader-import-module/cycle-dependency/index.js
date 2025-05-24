import lib from "./loader.js!./lib";

it("should cycle dependency works", () => {
	expect(lib).toBe(2);
});
