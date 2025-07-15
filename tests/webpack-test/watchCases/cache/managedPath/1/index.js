import { bar, foo } from "foo";

it("should invalidate resolving if inner node_modules appears", () => {
	expect(foo).toBe("foo");
	expect(bar).toBe("inner-bar");
});
