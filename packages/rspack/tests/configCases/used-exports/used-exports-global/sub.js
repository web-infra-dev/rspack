import { b, aUsed, bUsed } from "./lib";

it("should only import assets that included in chunks", () => {
	expect(b).toBe(3);
	expect(aUsed).toBe(true);
	expect(bUsed).toBe(true);
});
