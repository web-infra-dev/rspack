import value from "./src/index.mjs";

it("extension-alias should work", () => {
	expect(value).toBe("in ts");
});
