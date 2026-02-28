import { bar, foo } from "library";

it("should get default export from library (" + NAME + ")", function() {
	expect(bar).toBe("bar");
	expect(foo).toBe("foo");
});
