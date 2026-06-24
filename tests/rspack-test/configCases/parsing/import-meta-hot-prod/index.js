it("should transform import.meta.webpackHot and import.meta.hot to false", () => {
	let hot = false;
	if (import.meta.webpackHot) {
		hot = true;
		import.meta.webpackHot.accept();
	}

	expect(hot).toBe(false);

	let hotAlias = false;
	if (import.meta.hot) {
		hotAlias = true;
		import.meta.hot.accept();
	}

	expect(hotAlias).toBe(false);
});
