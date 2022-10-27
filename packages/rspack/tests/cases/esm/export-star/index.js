import * as lib from "./lib";

it("should work well with export star", function () {
	expect(lib.a).toBe(1);
	expect(lib.b).toBe(2);
});
