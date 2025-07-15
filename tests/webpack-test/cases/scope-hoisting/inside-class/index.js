import { Bar as FirstBar, Foo } from "./first"
import { Bar, Foo as SecondFoo } from "./second"

it("should renamed class reference in inner scope", function() {
	var a = new Foo().test();
	var b = new SecondFoo().test();
	expect(a).toBe(1);
	expect(b).toBe(2);
	expect(new FirstBar().test()).toBe(1);
	expect(new Bar().test()).toBe(2);
});
