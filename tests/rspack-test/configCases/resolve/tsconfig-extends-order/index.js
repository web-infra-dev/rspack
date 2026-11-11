import value from "@/value";

it("should prefer own options, then later entries in extends", () => {
	expect(value).toBe(EXPECTED);
});
