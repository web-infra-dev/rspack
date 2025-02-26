import { a } from "./a.js";
import { b } from "./b.js";

it("should compile", () => {
	expect(a + b).toBe(7);
});
