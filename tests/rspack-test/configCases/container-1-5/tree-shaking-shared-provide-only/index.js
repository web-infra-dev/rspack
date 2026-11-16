const mfScope = typeof __rspack_context === 'undefined' ? __webpack_require__ : __rspack_context;

it("should generate tree shaking shared fallback for provide-only shared modules", () => {
	const fallbacks =
		mfScope.federation.sharedFallback["provided-only"];

	expect(fallbacks).toBeTruthy();
	expect(fallbacks.map(([, version]) => version)).toEqual(["1.0.0"]);

	const [entry, , globalName] = fallbacks[0];
	const container = eval("require")(`./${entry}`)[globalName];
	expect(container.get()().value).toBe("provided-only");
});

it("should use configured version for local provide-only shared fallbacks", () => {
	const fallbacks =
		mfScope.federation.sharedFallback["local-provided"];

	expect(fallbacks).toBeTruthy();
	expect(fallbacks.map(([, version]) => version)).toEqual(["2.3.4"]);

	const [entry, , globalName] = fallbacks[0];
	const container = eval("require")(`./${entry}`)[globalName];
	expect(container.get()().value).toBe("local-provided");
});
