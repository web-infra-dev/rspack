import { a, aUsed, bUsed } from "./lib";

it("should use exports info per runtime ", () => {
	expect(a).toBe(3);
	expect(aUsed).toBe(true);
	expect(bUsed).toBe(false);
});
