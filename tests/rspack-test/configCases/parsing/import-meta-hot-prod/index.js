it('should transform import.meta.webpackHot to false', () => {
	let hot = false;
	if (import.meta.webpackHot) {
		hot = true;
    import.meta.webpackHot.accept();
  }

	expect(hot).toBe(false);
})
