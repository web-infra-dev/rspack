import * as lib from "./lib";

it("AMD local module should work well", function () {
	expect(lib.foo).toBe('foo');
	expect(typeof lib.add).toBe('function');
	expect(lib.add(1, 2)).toBe(3);
});
