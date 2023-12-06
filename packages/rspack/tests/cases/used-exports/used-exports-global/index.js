import { a, aUsed, bUsed } from "./lib";

it("should apply runtime opt", () => {
	expect(a).toBe(3);
	expect(aUsed).toBe(true);
	expect(bUsed).toBe(false);
});
