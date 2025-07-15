import { obj2 } from "./cjs.js";
import { obj } from "./module.js";

it("should have correct this in called function", () => {
	const r = obj.func();
	expect(r).toBe(obj);
	obj.test();
});

it("should have correct this in called function from commonjs", () => {
	const r = obj2.func();
	expect(r).toBe(obj2);
});
