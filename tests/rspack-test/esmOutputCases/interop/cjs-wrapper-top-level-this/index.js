import value from "./wrapped.cjs";

it("preserves CommonJS top-level this in the minimal wrapper", () => {
	expect(value).toBe(true);
});
