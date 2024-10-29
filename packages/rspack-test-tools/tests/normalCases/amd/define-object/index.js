import * as lib from "./lib";

it("define({...}) should work well", function () {
	expect(lib.a).toBe(1);
	expect(lib.b).toBe(2);
	expect(typeof lib.add).toBe('function');
	expect(lib.add(1, 2)).toBe(3);
});
