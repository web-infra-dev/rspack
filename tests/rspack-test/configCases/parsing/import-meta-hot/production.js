it("should transform import.meta.hot to false without HMR", () => {
	let hot = false;
	if (import.meta.hot) {
		hot = true;
		import.meta.hot.accept();
	}

	expect(hot).toBe(false);
});
