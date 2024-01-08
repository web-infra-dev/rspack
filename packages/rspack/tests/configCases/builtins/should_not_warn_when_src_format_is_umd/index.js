import { foo } from "./a.js";
foo;

it("should not warn when src format is umd", () => {
	expect(1).toBe(1);
});
