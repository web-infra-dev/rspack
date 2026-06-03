it("should generate tree shaking shared fallback for provide-only shared modules", () => {
	const fallbacks =
		__webpack_require__.federation.sharedFallback["provided-only"];

	expect(fallbacks).toBeTruthy();
	expect(fallbacks.map(([, version]) => version)).toEqual(["1.0.0"]);

	const [entry, , globalName] = fallbacks[0];
	const container = __non_webpack_require__(`./${entry}`)[globalName];
	expect(container.get()().value).toBe("provided-only");
});
