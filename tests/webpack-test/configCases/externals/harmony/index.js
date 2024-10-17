import external from "external";

it("should ESM import a dependency", function() {
	expect(external).toBe("abc");
});
