import value, { named } from "https://test.rspack.rs/value.js";

it("should bundle https imports for node target with buildHttp", () => {
	expect(value).toBe(42);
	expect(named).toBe(42);
});
