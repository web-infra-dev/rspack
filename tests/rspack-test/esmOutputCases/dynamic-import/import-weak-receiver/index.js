const included = require("./value");

it("should preserve the require receiver for a weak CommonJS import", async () => {
	expect(included).toBe("cjs");

	const namespace = await import(/* webpackMode: "weak" */ "./value");
	expect(namespace.default).toBe("cjs");
});

it("should preserve the require receiver for a direct namespace helper call", () => {
	const namespace = __webpack_require__.t(require.resolveWeak("./value"), 1);
	expect(namespace.default).toBe("cjs");
});
