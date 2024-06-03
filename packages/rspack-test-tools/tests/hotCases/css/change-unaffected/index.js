it("css hmr", (done) => {
	NEXT(require("../../update")(done, true, async () => {
		const styles = await import('./entry.js')
		expect(styles['default']).toHaveProperty("bar");
		done();
	}));
}, 1000);
