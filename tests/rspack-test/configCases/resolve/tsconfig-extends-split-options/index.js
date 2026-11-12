import value from "@/value";

it("should resolve paths against the final merged baseUrl", () => {
	expect(value).toBe(EXPECTED);
});
