import value from "@alias/value";

it("should locate the package of a tsconfig extends with the package map", () => {
	expect(value).toBe("src/value");
});
