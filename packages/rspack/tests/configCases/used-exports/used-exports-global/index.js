import { a, aUsed, bUsed } from "./lib";

it("should use global used for exports", () => {
	expect(a).toBe(3);
	expect(aUsed).toBe(true);
	expect(bUsed).toBe(true);
});
