const included = require("./value");

it("should preserve the require receiver for a weak CommonJS import", async () => {
	expect(included).toBe("cjs");

	const namespace = await import(/* webpackMode: "weak" */ "./value");
	expect(namespace.default).toBe("cjs");
});
