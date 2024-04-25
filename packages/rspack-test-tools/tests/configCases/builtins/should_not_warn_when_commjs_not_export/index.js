import test from "./a.js";

test;

it("should not warn when commonjs not export", () => {
	expect(1).toBe(1);
});
