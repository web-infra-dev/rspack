import { a, b, c, d } from "./test.d.ts";

it("should can import d.ts", function () {
	expect(a).toBeUndefined();
	expect(b).toBeUndefined();
	expect(c).toBeUndefined();
	expect(d).toBeUndefined();
});
