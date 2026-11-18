import value from "@value";

it("should recover after fixing only the extended tsconfig", () => {
	expect(value).toBe(42);
});
