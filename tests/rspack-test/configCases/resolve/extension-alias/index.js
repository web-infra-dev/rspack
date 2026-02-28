import value from "./src/index.mjs";

it("extension-alias should work", () => {
	expect(value).toBe("in ts");
});

it("extension-alias should report meaningful error message", () => {
	try {
		require("./src/m1.mjs")
	} catch (e) {
		expect(e.message).toContain("Cannot resolve 'm1.mjs' for extension aliases 'm1.mts'");
	}
});
