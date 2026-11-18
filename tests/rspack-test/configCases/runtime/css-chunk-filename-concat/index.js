it("generates CSS chunk filenames with the chunk ID and content hash", async () => {
	const style = await import(/* webpackChunkName: "12345" */ "./style.css");
	expect(style).toEqual(nsObj({}));
});
