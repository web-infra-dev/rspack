import lib from "./loader.mjs!./lib";

it("should cycle dependency works", () => {
	expect(lib).toBe(2);
});
