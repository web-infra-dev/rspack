import { getValue, setValue, value } from "./foo.js";
import wrapped from "./wrapped.js";

it("should hoist analyzable CommonJS and wrap the rest", () => {
	expect(value).toBe(1);
	expect(getValue()).toBe(1);

	setValue(2);

	expect(value).toBe(2);
	expect(getValue()).toBe(2);
	expect(wrapped.value).toBe(42);
});
