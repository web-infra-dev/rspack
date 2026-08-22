import { value } from "./foo.js";
import { esmValue } from "./esm.js";

it("should keep an eligible CommonJS module wrapped when opted out", () => {
	expect(value).toBe(1);
	expect(esmValue).toBe(2);
});
