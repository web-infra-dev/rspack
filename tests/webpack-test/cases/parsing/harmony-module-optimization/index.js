import { check as c1, value as v1 } from "./module";
import { check as c2, value as v2 } from "./module-normal";

it("should allow to optimize exports in modules using 'module'", () => {
	expect(v1).toBe(42);
	expect(v2).toBe(42);
	expect(c1).toBe(c2);
});
