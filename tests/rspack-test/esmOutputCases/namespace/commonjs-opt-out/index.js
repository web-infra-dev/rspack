import { value } from "./foo.js";

it("should keep an eligible CommonJS module wrapped when opted out", () => {
	expect(value).toBe(1);
});
