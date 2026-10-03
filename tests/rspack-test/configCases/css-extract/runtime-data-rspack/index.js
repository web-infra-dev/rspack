it("should mark links created by the extract CSS runtime", async () => {
	const before = Array.from(document.getElementsByTagName("link"));
	await import(/* webpackChunkName: "style" */ "./style.css");
	const created = Array.from(document.getElementsByTagName("link")).filter(link => !before.includes(link));
	expect(created).toHaveLength(1);
	expect(created[0].getAttribute("data-rspack")).toBe(UNIQUE_NAME + ":mini-css-chunk-style");
});
