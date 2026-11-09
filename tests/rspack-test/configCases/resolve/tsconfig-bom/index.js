import value from "bom-value";

it("should resolve imports with a UTF-8 BOM tsconfig", () => {
	expect(value).toBe("bom");
});
