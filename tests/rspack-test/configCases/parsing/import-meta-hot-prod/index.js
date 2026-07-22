it('should transform import.meta.webpackHot to false', () => {
	let hot = false;
	if (import.meta.webpackHot) {
		hot = true;
    import.meta.webpackHot.accept();
  }

	expect(hot).toBe(false);
})

it("should short-circuit optional calls on import.meta.webpackHot", () => {
	let callbackArgumentEvaluated = false;
	const createCallback = () => {
		callbackArgumentEvaluated = true;
		return () => {};
	};

	expect(
		import.meta.webpackHot?.dispose(createCallback())
	).toBeUndefined();
	expect(callbackArgumentEvaluated).toBe(false);
});
