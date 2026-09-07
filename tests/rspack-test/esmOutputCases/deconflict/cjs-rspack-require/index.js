import "./require.cjs";

const value = __webpack_require__("./require.cjs");

it("should avoid a CJS rspackRequire factory parameter conflict", () => {
	expect(value).toBe(42);
});
